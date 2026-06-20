use egui::{Color32, FontId, Pos2, Rect, Stroke, Vec2};
use egui::text::LayoutJob;

use crate::theme::AppTheme;
use snemcore::Snemulator;
use snemcore::ssmp::sdsp::SuperDSP;
use super::append;

// ─── Layout constants ─────────────────────────────────────────────────────────

const GRID_HEIGHT: f32 = 150.0;
/// Number of sub-sample steps used to draw the gaussian curve across one sample interval.
const CURVE_SUBSTEPS: usize = 16;
/// Minimum pixel distance between amplitude grid lines before we stop drawing them.
const MIN_GRID_LINE_SPACING: f32 = 4.0;
/// Radius of sample dot circles.
const DOT_RADIUS: f32 = 4.0;

// ─── Struct ───────────────────────────────────────────────────────────────────

pub struct SdspBrrTab {
    voice_open: [bool; 8],
}

impl SdspBrrTab {
    pub fn new() -> Self {
        Self { voice_open: [false; 8] }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, core: &mut Snemulator, t: &AppTheme) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for v in 0..8usize {
                let vr = &core.ssmp.voice_regs[v];

                let header_job = self.format_header(t, v, vr);

                let resp = egui::CollapsingHeader::new(header_job)
                    .id_salt(format!("brr_voice_{v}"))
                    .open(Some(self.voice_open[v]))
                    .show(ui, |ui| {
                        // Snapshot the fields we need — avoids holding borrow into painter call.
                        let buf        = core.ssmp.voice_regs[v].brr_sample_buffer;
                        let interp_idx = core.ssmp.voice_regs[v].prev_interpolation_idx;
                        let shift      = Self::current_shift(&core.ssmp.voice_regs[v]);
                        let group_step = core.ssmp.voice_regs[v].brr_group_step;

                        ui.add_space(4.0);
                        let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), GRID_HEIGHT));
                        let dot_positions = Self::paint_grid(ui, t, rect, &buf, interp_idx, shift, group_step);

                        // ── Hover tooltips for each BRR sample dot ───────────
                        for (col, (&raw, (cx, cy))) in buf.iter().zip(dot_positions.iter()).enumerate() {
                            let signed = ((raw << 1) as i16) >> 1;
                            let dot_rect = Rect::from_center_size(
                                Pos2::new(*cx, *cy),
                                Vec2::splat(DOT_RADIUS * 2.0 + 4.0),
                            );
                            let resp = ui.allocate_rect(dot_rect, egui::Sense::hover());
                            if resp.hovered() {
                                egui::show_tooltip_at_pointer(ui.ctx(), ui.layer_id(), egui::Id::new(("brr_tip", v, col)), |ui| {
                                    ui.label(format!("buf[{col}]"));
                                    ui.label(format!("Raw:    ${raw:04X}"));
                                    ui.label(format!("Signed: {signed}"));
                                });
                            }
                        }

                        ui.add_space(4.0);
                    });

                if resp.header_response.clicked() {
                    self.voice_open[v] = !self.voice_open[v];
                }

                resp.header_response.context_menu(|ui| {
                    self.render_context_menu(ui, v);
                });
            }
        });
    }

    // ── Header ────────────────────────────────────────────────────────────────

    fn format_header(&self, t: &AppTheme, v: usize, vr: &snemcore::ssmp::sdsp::voices::VoiceRegs) -> LayoutJob {
        let mut job = LayoutJob::default();
        let mono = FontId::monospace(13.0);

        append(&mut job, &format!("VOICE {v}  "), mono.clone(), t.accent);
        append(&mut job, "Shift: ", mono.clone(), t.text_secondary);
        append(&mut job, &format!("{}  ", Self::current_shift(vr)), mono.clone(), t.syntax_number);
        append(&mut job, "BRR Addr: ", mono.clone(), t.text_secondary);
        append(&mut job, &format!("${:04X}  ", vr.brr_group_addr), mono.clone(), t.syntax_address);
        append(&mut job, "Group Step: ", mono.clone(), t.text_secondary);
        append(&mut job, &format!("{}", vr.brr_group_step), mono.clone(), t.syntax_number);

        job
    }

    // ── Grid painter ──────────────────────────────────────────────────────────

    fn paint_grid(
        ui: &egui::Ui,
        t: &AppTheme,
        rect: Rect,
        buf: &[u16; 12],
        interp_idx: usize,
        shift: u8,
        group_step: usize,
    ) -> [(f32, f32); 12] {
        let painter = ui.painter_at(rect);
        let w = rect.width();
        let h = rect.height();
        let mid_y = rect.center().y;

        // ── Background ───────────────────────────────────────────────────────
        painter.rect_filled(rect, t.corner_radius as f32, t.bg_tertiary);

        // Column width for each of the 12 buffer samples.
        let col_w = w / 12.0;

        // Highlight columns belonging to the current BRR group.
        // brr_group_step is 0-based; each step covers 4 samples.
        // The "current four" samples are always at indices 4..=7 in the buffer.
        let current_group_start_col = 4usize;
        for col in current_group_start_col..current_group_start_col + 4 {
            let x = rect.left() + col as f32 * col_w;
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(col_w, h)),
                0.0,
                t.bg_elevated,
            );
        }

        // ── Amplitude grid lines ─────────────────────────────────────────────
        // shift determines the amplitude range: max amplitude = 1 << shift (clamped to i16 range).
        // We draw horizontal lines at every power-of-two step for the current shift.
        let amplitude_range = (1i32 << shift.min(12)) as f32;
        // pixels per unit amplitude
        let px_per_unit = (h * 0.5) / amplitude_range;
        // Draw lines at intervals that keep spacing above the minimum.
        // Start at single-unit steps and double until spacing is large enough.
        let mut step_units = 1i32;
        while (step_units as f32 * px_per_unit) < MIN_GRID_LINE_SPACING {
            step_units *= 2;
        }
        {
            let mut amp = 0i32;
            while amp <= amplitude_range as i32 {
                for &sign in &[1, -1] {
                    let y = mid_y - (amp as f32 * sign as f32) * px_per_unit;
                    if y >= rect.top() && y <= rect.bottom() {
                        painter.line_segment(
                            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                            Stroke::new(if amp == 0 { 1.0 } else { 0.5 }, t.border),
                        );
                    }
                    if amp == 0 { break; } // only draw zero line once
                }
                amp += step_units;
            }
        }

        // ── Group boundary lines ─────────────────────────────────────────────
        for col in (0..=12).step_by(4) {
            let x = rect.left() + col as f32 * col_w;
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, t.border),
            );
        }

        // ── Helper: sign-extend 15-bit BRR sample stored as u16 ─────────────
        let brr_sign = |raw: u16| -> i16 { ((raw << 1) as i16) >> 1 };

        // ── Helper: map a sample value to a y coordinate ─────────────────────
        let sample_y = |s: i16| -> f32 {
            mid_y - (s as f32 * px_per_unit).clamp(-h * 0.5, h * 0.5)
        };

        // ── Gaussian interpolation curve ─────────────────────────────────────
        // We evaluate across the visible 12 sample columns in fine steps.
        // For each sub-step position, compute the fractional pitch index and
        // apply the SNES gaussian kernel using the same math as the DSP.
        let gauss = &SuperDSP::GAUSS_LOOKUP_TABLE;
        let total_steps = 12 * CURVE_SUBSTEPS;
        let mut curve_points: Vec<Pos2> = Vec::with_capacity(total_steps);

        for step in 0..total_steps {
            // Fractional position within the 12-sample range, in units of
            // interpolation_idx (0x0000..0x3000 for 12 samples).
            let frac_pos = step as f32 / CURVE_SUBSTEPS as f32; // 0.0 .. 12.0
            let coarse = frac_pos.floor() as usize; // 0..11
            let frac_unit = (frac_pos.fract() * 0x1000 as f32) as usize; // 0..0xFFF

            // Gaussian table indices
            let g_frac = (frac_unit >> 4) & 0xFF;
            let w0 = gauss[0xFF  - g_frac] as i32;
            let w1 = gauss[0x1FF - g_frac] as i32;
            let w2 = gauss[0x100 + g_frac] as i32;
            let w3 = gauss[0x000 + g_frac] as i32;

            // The four samples around this position, mirroring the DSP's indexing.
            // coarse maps to the buffer index of the "current" sample.
            // We clamp to keep indices in [0, 11].
            let s  = |i: usize| -> i32 { brr_sign(buf[i.clamp(0, 11)]) as i32 };
            let i0 = coarse.saturating_sub(1);
            let i1 = coarse;
            let i2 = (coarse + 1).min(11);
            let i3 = (coarse + 2).min(11);

            let out = (w0 * s(i0) + w1 * s(i1) + w2 * s(i2) + w3 * s(i3)) >> 11;
            let out = out.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

            let x = rect.left() + frac_pos / 12.0 * w;
            curve_points.push(Pos2::new(x, sample_y(out)));
        }

        if curve_points.len() >= 2 {
            painter.add(egui::Shape::line(curve_points, Stroke::new(1.5, t.success)));
        }

        // ── Raw BRR sample dots ───────────────────────────────────────────────
        // Painting happens here; hover tooltips are handled after paint_grid returns
        // via per-column sense rects allocated by the caller.
        let mut dot_positions = [(0.0f32, 0.0f32); 12];
        for (col, &raw) in buf.iter().enumerate() {
            let s  = brr_sign(raw);
            let cx = rect.left() + (col as f32 + 0.5) * col_w;
            let cy = sample_y(s);
            dot_positions[col] = (cx, cy);
            // Vertical tick to zero line
            painter.line_segment(
                [Pos2::new(cx, cy), Pos2::new(cx, mid_y)],
                Stroke::new(1.0, t.text_disabled),
            );
            painter.circle(cy_pos(cx, cy), DOT_RADIUS, Color32::TRANSPARENT,
                Stroke::new(1.5, t.syntax_number));
        }

        // ── Interpolation position marker ─────────────────────────────────────
        // The coarse index into the buffer is (interp_idx >> 12), offset by 4
        // per the DSP's buffer layout.
        let coarse_idx = (interp_idx >> 12) & 0x3;
        let frac       = (interp_idx >> 4) & 0xFF;

        // Evaluate the gaussian output at the current interpolation position.
        let gauss_w0 = gauss[0xFF  - frac] as i32;
        let gauss_w1 = gauss[0x1FF - frac] as i32;
        let gauss_w2 = gauss[0x100 + frac] as i32;
        let gauss_w3 = gauss[0x000 + frac] as i32;

        let s = |i: usize| -> i32 { brr_sign(buf[i]) as i32 };
        let interp_out = (
              gauss_w0 * s(4 + coarse_idx - 2)  // wrapping handled below
            + gauss_w1 * s(4 + coarse_idx - 1)
            + gauss_w2 * s(4 + coarse_idx)
            + gauss_w3 * s(4 + coarse_idx + 1)
        ) >> 11;
        let interp_out = interp_out.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

        // X position: the current sample slot is at buffer index 4 + coarse_idx,
        // plus the fractional part within that column.
        let col_pos = (4 + coarse_idx) as f32 + (frac as f32 / 255.0);
        let marker_x = rect.left() + (col_pos / 12.0) * w;
        let marker_y = sample_y(interp_out);

        // Vertical line spanning the full height to make the position easy to spot.
        painter.line_segment(
            [Pos2::new(marker_x, rect.top()), Pos2::new(marker_x, rect.bottom())],
            Stroke::new(1.0, t.accent_muted),
        );
        // Filled dot at the interpolated output value.
        painter.circle_filled(cy_pos(marker_x, marker_y), DOT_RADIUS, t.accent);

        // ── Border ────────────────────────────────────────────────────────────
        painter.rect_stroke(rect, t.corner_radius as f32,
            Stroke::new(1.0, t.border), egui::StrokeKind::Outside);

        dot_positions
    }

    // ── Context menu ──────────────────────────────────────────────────────────

    fn render_context_menu(&mut self, ui: &mut egui::Ui, v: usize) {
        ui.label(format!("Voice {v}"));
        ui.separator();
        // Add context menu items here
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Extracts the BRR shift for the current group from the voice state.
    /// BRR headers encode shift in the high nibble of the group header byte.
    /// We derive it from brr_group_addr; if unavailable, fall back to 0.
    fn current_shift(vr: &snemcore::ssmp::sdsp::voices::VoiceRegs) -> u8 {
        // The shift is embedded in the BRR block header. Since we don't store
        // the raw header here, we approximate from the sample range actually
        // present in the buffer: find the largest absolute value and back-derive
        // the shift tier. This is an approximation — replace with a stored
        // header field if one becomes available.
        let max_abs = vr.brr_sample_buffer.iter()
            .map(|&s| (s as i16).unsigned_abs())
            .max()
            .unwrap_or(0);
        if max_abs == 0 { return 0; }
        let bits = u16::BITS - max_abs.leading_zeros();
        bits.saturating_sub(1) as u8
    }
}

// ── Free helpers ──────────────────────────────────────────────────────────────

#[inline]
fn cy_pos(x: f32, y: f32) -> Pos2 { Pos2::new(x, y) }