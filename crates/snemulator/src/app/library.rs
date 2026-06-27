use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use egui::{Ui, Vec2};

use crate::app::theme::AppTheme;
use crate::app::thumbnail_fetcher::{self, ThumbnailResult};
use crate::app::{AppAction, settings::Settings};
use crate::app::rom_paths::{RomManifest, RomPaths};

const MAX_ROM_DIR_SEARCH_DEPTH: usize = 3;

pub enum ThumbnailState {
    Loading,
    Ready(PathBuf),
    NotFound,
}

pub struct LibraryEntry {
    pub path: PathBuf,
    pub display_name: String,
    pub has_sav: bool,
    pub used_slots: Vec<u32>, // slot indices that exist on disk
    pub last_played: Option<u64>,
    pub play_time_secs: u64,
    pub thumbnail: ThumbnailState,
}

pub struct LibraryView {
    pub entries: Vec<LibraryEntry>,
    selected: Option<usize>,
    thumbnail_rx: Option<Receiver<ThumbnailResult>>,
}

impl LibraryView {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected: None,
            thumbnail_rx: None,
        }
    }

    pub fn update_entry(&mut self, path: &PathBuf, settings: &Settings) {
        let Some(entry) = self.entries.iter_mut().find(|e| e.path == *path) else { return };
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let manifest = RomPaths::find_manifest_by_stem(stem);
        let paths = RomPaths::new(stem);

        entry.last_played = manifest.as_ref().and_then(|m| m.last_played);
        entry.play_time_secs = manifest.as_ref().map(|m| m.play_time_secs).unwrap_or(0);
        entry.has_sav = paths.as_ref().map(|p| p.sav_path().exists()).unwrap_or(false);
        entry.used_slots = paths.map(|p| {
            (0..settings.save_state_slots)
                .filter(|&slot| p.state_path(slot).exists())
                .collect()
        }).unwrap_or_default();
        entry.thumbnail = match manifest.as_ref().and_then(|m| m.thumbnail_path.clone()) {
            Some(p) => ThumbnailState::Ready(p),
            None    => ThumbnailState::NotFound,
        };
    }

    /// Re-scan the library folder. Call this when the folder changes or on startup.
    pub fn scan(&mut self, settings: &Settings) {
        self.entries.clear();
        self.selected = None;
        let Some(lib_dir) = &settings.roms_library_dir else { return };
        Self::scan_dir(lib_dir, 0, settings, &mut self.entries);
        self.entries.sort_by(|a, b| a.display_name.cmp(&b.display_name));

        // Collect stems that need thumbnails (all Loading entries after scan)
        let stems: Vec<(String, PathBuf)> = self.entries.iter()
            .filter(|e| matches!(e.thumbnail, ThumbnailState::Loading))
            .filter_map(|e| {
                let stem = e.path.file_stem()?.to_str()?.to_string();
                Some((stem, e.path.clone()))
            })
            .collect();

        log::debug!("Stems found with no thumbnails: {}", stems.len());

        if !stems.is_empty() {
            let (tx, rx) = mpsc::channel();
            self.thumbnail_rx = Some(rx);
            thumbnail_fetcher::spawn_thumbnail_resolver(stems, tx);
            log::debug!("Spawned thumbnail fetcher thread");
        }
    }

    fn scan_dir(dir: &PathBuf, depth: usize, settings: &Settings, entries: &mut Vec<LibraryEntry>) {
        let Ok(read_dir) = std::fs::read_dir(dir) else { return };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() && depth < MAX_ROM_DIR_SEARCH_DEPTH {
                Self::scan_dir(&path, depth + 1, settings, entries);
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "sfc" | "smc") { continue; }

            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let paths = RomPaths::new(&stem);

            let manifest = match RomPaths::find_manifest_by_stem(&stem) {
                Some(m) => m,
                None => {
                    let bytes: Option<Vec<u8>> = std::fs::read(&path).ok();

                    // First time seeing this ROM — derive title from header if possible
                    let display_name = bytes.as_ref()
                        .and_then(|bytes| snemcore::cartridge::get_rom_title(&bytes))
                        .unwrap_or_else(|| stem.clone());

                    let crc: u32 = bytes.as_ref()
                        .and_then(|bytes| Some(crc32fast::hash(&bytes)))
                        .unwrap_or_default();

                    let m = RomManifest {
                        rom_crc: crc,
                        display_name,
                        ..Default::default()
                    };

                    if let Some(p) = &paths {
                        let _ = p.ensure_dirs();
                        p.write_manifest(&m);
                    }

                    m
                }
            };

            let has_sav = paths.as_ref().map(|p| p.sav_path().exists()).unwrap_or(false);
            let used_slots = paths.map(|p| {
                (0..settings.save_state_slots)
                    .filter(|&slot| p.state_path(slot).exists())
                    .collect()
            }).unwrap_or_default();

            entries.push(LibraryEntry {
                path,
                display_name: manifest.display_name,
                has_sav,
                used_slots,
                last_played: manifest.last_played,
                play_time_secs: manifest.play_time_secs,
                thumbnail: match &manifest.thumbnail_path {
                    Some(p) => ThumbnailState::Ready(p.clone()),
                    None    => ThumbnailState::Loading,
                },
            });
        }
    }

    pub fn render(&mut self, ui: &mut Ui, app_theme: &AppTheme) -> Option<AppAction> {
        if let Some(rx) = &self.thumbnail_rx {
            'receive_thumbnails: loop {
                match rx.try_recv() {
                    Ok(result) => {
                        log::debug!("Received thumbnail: found={}, path={:?}", result.path.is_some(), result.path);
        
                        if let Some(entry) = self.entries.iter_mut()
                            .find(|e| e.path.file_stem().and_then(|s| s.to_str()) == Some(&result.stem))
                        {
                            entry.thumbnail = match result.path {
                                Some(p) => ThumbnailState::Ready(p),
                                None    => ThumbnailState::NotFound,
                            };
                        }
                    }
                    Err(TryRecvError::Empty) => { break 'receive_thumbnails; }
                    Err(TryRecvError::Disconnected) => {
                        self.thumbnail_rx = None;

                        for entry in self.entries.iter_mut() {
                            if matches!(entry.thumbnail, ThumbnailState::Loading) {
                                entry.thumbnail = ThumbnailState::NotFound;
                            }
                        }

                        break 'receive_thumbnails;
                    }
                }

            }
        }

        let mut action: Option<AppAction> = None;

        Self::render_header(ui, app_theme);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            for (i, entry) in self.entries.iter().enumerate() {
                let is_selected = self.selected == Some(i);
                
                let response = Self::render_entry(ui, entry, is_selected, app_theme);
                
                if response.double_clicked() {
                    action = Some(AppAction::LoadRomFromPath(entry.path.clone()));
                } else if response.clicked() {
                    self.selected = Some(i);
                }

                #[cfg(feature = "debug")]
                response.context_menu(|ui| {
                    if ui.button("Debug").clicked() {
                        action = Some(AppAction::OpenDebug(Some(entry.path.clone())));
                        ui.close();
                    }
                });
            }
        });
        action
    }

    fn render_header(ui: &mut Ui, theme: &AppTheme) {
        let available_width = ui.available_width();
        let row_height = 24.0;

        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(available_width, row_height),
            egui::Sense::hover(),
        );

        if !ui.is_rect_visible(rect) { return; }

        let painter = ui.painter();

        painter.rect_filled(rect, 0.0, theme.bg_secondary);
        painter.line_segment(
            [egui::pos2(rect.min.x, rect.max.y - 0.5), egui::pos2(rect.max.x, rect.max.y - 0.5)],
            egui::Stroke::new(0.5, theme.border),
        );

        let margin = 12.0;
        let thumb_size = 44.0;
        let col_sav_w    = 40.0;
        let col_states_w = 80.0;
        let col_played_w = 110.0;
        let col_time_w   = 90.0;

        let text_x = margin + thumb_size + 10.0;
        let mid_y = rect.center().y;

        let label = |painter: &egui::Painter, x: f32, anchor: egui::Align2, text: &str| {
            painter.text(
                egui::pos2(x, mid_y),
                anchor,
                text,
                egui::FontId::proportional(10.0),
                theme.text_disabled,
            );
        };

        label(&painter, text_x, egui::Align2::LEFT_CENTER, "NAME");

        let mut col_right_x = rect.max.x - margin;

        col_right_x -= col_time_w;
        label(&painter, col_right_x + col_time_w / 2.0, egui::Align2::CENTER_CENTER, "TIME PLAYED");

        col_right_x -= col_played_w;
        label(&painter, col_right_x + col_played_w / 2.0, egui::Align2::CENTER_CENTER, "LAST PLAYED");

        col_right_x -= col_states_w;
        label(&painter, col_right_x + col_states_w / 2.0, egui::Align2::CENTER_CENTER, "SAVE STATES");

        col_right_x -= col_sav_w;
        label(&painter, col_right_x + col_sav_w / 2.0, egui::Align2::CENTER_CENTER, "SAV");
    }

    fn render_entry(ui: &mut Ui, entry: &LibraryEntry, is_selected: bool, theme: &AppTheme) -> egui::Response {
        const ROW_HEIGHT: f32 = 116.0;
        const THUMBNAIL_SIZE: f32 = 96.0;
        const THUMBNAIL_MARGIN: f32 = 12.0;

        let available_width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, ROW_HEIGHT),
            egui::Sense::click(),
        );

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter();
        let cr = theme.corner_radius as f32;

        // Background
        if is_selected {
            painter.rect_filled(rect, 0.0, theme.bg_elevated);
            painter.rect_filled(
                egui::Rect::from_min_size(rect.min, Vec2::new(3.0, ROW_HEIGHT)),
                0.0,
                theme.accent,
            );
        } else if response.hovered() {
            painter.rect_filled(rect, 0.0, theme.bg_secondary);
        }

        // Separator
        painter.line_segment(
            [egui::pos2(rect.min.x, rect.max.y - 0.5), egui::pos2(rect.max.x, rect.max.y - 0.5)],
            egui::Stroke::new(0.5, theme.border),
        );

        // Thumbnail
        let thumb_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + THUMBNAIL_MARGIN, rect.min.y + (ROW_HEIGHT - THUMBNAIL_SIZE) / 2.0),
            Vec2::splat(THUMBNAIL_SIZE),
        );
        painter.rect_filled(thumb_rect, cr, theme.bg_tertiary);
        painter.rect_stroke(
            thumb_rect, 
            cr, 
            egui::Stroke::new(1.0, theme.border),
            egui::StrokeKind::Outside,
        );
        match &entry.thumbnail {
            ThumbnailState::Loading => {
                // Spinning arc loader
                let center = thumb_rect.center();
                let radius = 14.0;
                let t = ui.ctx().input(|i| i.time) as f32;
                let start_angle = t * 2.5;
                let sweep = std::f32::consts::PI * 1.4;

                let steps = 32usize;
                let points: Vec<egui::Pos2> = (0..=steps)
                    .map(|i| {
                        let angle = start_angle + (i as f32 / steps as f32) * sweep;
                        egui::pos2(
                            center.x + radius * angle.cos(),
                            center.y + radius * angle.sin(),
                        )
                    })
                    .collect();

                for pair in points.windows(2) {
                    let alpha = (pair[0].x - center.x + radius) / (2.0 * radius);
                    painter.line_segment(
                        [pair[0], pair[1]],
                        egui::Stroke::new(2.5, theme.accent.linear_multiply(alpha.clamp(0.2, 1.0))),
                    );
                }

                ui.ctx().request_repaint();
            }
            ThumbnailState::Ready(thumb_path) => {
                let path_str = thumb_path.to_string_lossy().replace('\\', "/");
                let uri = format!("file://{}", path_str);
                ui.put(thumb_rect, egui::Image::new(uri)
                    .fit_to_exact_size(Vec2::splat(THUMBNAIL_SIZE))
                    .corner_radius(cr));
            }
            ThumbnailState::NotFound => {
                let initial = entry.display_name.chars().next()
                    .and_then(|c| c.to_uppercase().next())
                    .unwrap_or('?');
                painter.text(
                    thumb_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    initial.to_string(),
                    egui::FontId::proportional(48.0),
                    theme.text_muted,
                );
            }
        }

        let painter = ui.painter();

        // Column layout
        let col_sav_w    = 40.0;
        let col_states_w = 80.0;
        let col_played_w = 110.0;
        let col_time_w   = 90.0;

        let text_x = thumb_rect.max.x + 10.0;
        let mid_y = rect.center().y;

        // Title + subtitle
        painter.text(
            egui::pos2(text_x, mid_y - 9.0),
            egui::Align2::LEFT_CENTER,
            &entry.display_name,
            egui::FontId::proportional(13.0),
            if is_selected { theme.text_primary } else { theme.text_secondary },
        );
        let stem = entry.path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem != entry.display_name {
            painter.text(
                egui::pos2(text_x, mid_y + 9.0),
                egui::Align2::LEFT_CENTER,
                stem,
                egui::FontId::proportional(10.0),
                theme.text_muted,
            );
        }

        // Right columns
        let mut col_right_x = rect.max.x - THUMBNAIL_MARGIN;

        col_right_x -= col_time_w;
        painter.text(
            egui::pos2(col_right_x + col_time_w / 2.0, mid_y),
            egui::Align2::CENTER_CENTER,
            format_play_time(entry.play_time_secs),
            egui::FontId::proportional(11.0),
            theme.text_muted,
        );

        col_right_x -= col_played_w;
        painter.text(
            egui::pos2(col_right_x + col_played_w / 2.0, mid_y),
            egui::Align2::CENTER_CENTER,
            entry.last_played.map(format_timestamp).unwrap_or_else(|| "Never".to_string()),
            egui::FontId::proportional(11.0),
            theme.text_muted,
        );

        col_right_x -= col_states_w;
        painter.text(
            egui::pos2(col_right_x + col_states_w / 2.0, mid_y),
            egui::Align2::CENTER_CENTER,
            if entry.used_slots.is_empty() {
                "—".to_string()
            } else {
                format!("{} state{}", entry.used_slots.len(), if entry.used_slots.len() == 1 { "" } else { "s" })
            },
            egui::FontId::proportional(11.0),
            theme.text_muted,
        );

        col_right_x -= col_sav_w;
        if entry.has_sav {
            let badge_center = egui::pos2(col_right_x + col_sav_w / 2.0, mid_y);
            let badge_rect = egui::Rect::from_center_size(badge_center, Vec2::new(28.0, 16.0));
            painter.rect_filled(badge_rect, theme.widget_corner_radius as f32, theme.success.linear_multiply(0.25));
            painter.rect_stroke(
                badge_rect, 
                theme.widget_corner_radius as f32, 
                egui::Stroke::new(0.5, theme.success.linear_multiply(0.6)),
                egui::StrokeKind::Outside,
            );
            painter.text(
                badge_center,
                egui::Align2::CENTER_CENTER,
                "SAV",
                egui::FontId::proportional(9.0),
                theme.success,
            );
        }

        response
    }

}
fn format_play_time(secs: u64) -> String {
    if secs == 0 { return "—".to_string(); }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

fn format_timestamp(unix_secs: u64) -> String {
    // Without a date library we do a rough conversion.
    // Good enough for display; swap for chrono if you add it.
    let days_since_epoch = unix_secs / 86400;
    // Rough Gregorian: good to ~2100
    let year  = 1970 + days_since_epoch / 365;
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30).min(11) + 1;
    let day   = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}