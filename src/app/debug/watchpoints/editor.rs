use crate::app;
use crate::app::debug::watchpoints::notes;
use crate::app::debug::watchpoints::types::HWVAL_NAMES;
use crate::app::debug::watchpoints::types::*;
use crate::app::utils::monospace_text;
use crate::core::snemcore;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::{HashMap, HashSet};

// ── Constants ────────────────────────────────────────────────────────────────

const PORT_RADIUS: f32 = 6.0;
const WIRE_THICKNESS: f32 = 2.5;
/// Radius (in screen pixels) within which a port registers a hit.
const PORT_HIT_RADIUS: f32 = 20.0;
const ZOOM_MIN: f32 = 0.50;
const ZOOM_MAX: f32 = 2.00;
const ZOOM_STEP: f32 = 0.10;
const PAN_MAX_X: f32 = 1000.0;
const PAN_MAX_Y: f32 = 1000.0;

// ── Selection ─────────────────────────────────────────────────────────────────

/// What is currently selected. Mutually exclusive states — an enum is the
/// right tool to prevent impossible combinations.
#[derive(Debug, Default)]
enum Selection {
    #[default]
    None,
    /// A single node, possibly being dragged.
    SingleNode(NodeId),
    /// Multiple nodes selected via marquee.
    MultiNode(HashSet<NodeId>),
    /// An index into `Graph::wires`, awaiting deletion.
    Wire(usize),
}

impl Selection {
    fn contains_node(&self, id: NodeId) -> bool {
        match self {
            Selection::SingleNode(n) => *n == id,
            Selection::MultiNode(set) => set.contains(&id),
            _ => false,
        }
    }

    fn node_ids(&self) -> Vec<NodeId> {
        match self {
            Selection::SingleNode(id) => vec![*id],
            Selection::MultiNode(set) => set.iter().copied().collect(),
            _ => vec![],
        }
    }
}

// ── Drag state machine ────────────────────────────────────────────────────────

enum DragState {
    Idle,
    /// Moving one or more nodes. Stores per-node offsets (cursor - node.pos at
    /// drag start) so each node tracks the cursor without jumping.
    DraggingNodes(Vec<(NodeId, Vec2)>),
    /// Drawing a new wire from an output port.
    DraggingWire {
        from: Port,
        /// Current tip position in canvas space.
        cursor: Pos2,
    },
    /// Rubber-band rectangle selection. `anchor` is the fixed corner.
    DraggingMarquee {
        anchor: Pos2,
        current: Pos2,
    },
    CreatingNode(NodeId),
    /// Creating a watchpoint and accompanying break node.
    CreatingWatchpoint((NodeId, NodeId)),
}

// ── Editor ────────────────────────────────────────────────────────────────────

pub struct Editor {
    pub graph: Graph,
    drag: DragState,
    /// Canvas pan offset in screen pixels.
    pan: Vec2,
    /// Zoom level: canvas units per screen pixel (>1 = zoomed in).
    zoom: f32,
    /// Cached signal values from the last evaluation pass.
    signals: HashMap<Port, bool>,
    selection: Selection,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            drag: DragState::Idle,
            pan: Vec2::ZERO,
            zoom: 1.0,
            signals: HashMap::new(),
            selection: Selection::None,
        }
    }

    pub fn update_watchpoints(&mut self, compiled_wps: &CompiledGraph) {
        for op in compiled_wps.iter() {
            match op {
                FastOp::CounterRisingEdge {
                    count, fired, id, ..
                }
                | FastOp::CounterHigh {
                    count, fired, id, ..
                } => {
                    let node = self.graph.nodes.get_mut(*id).unwrap();

                    match &mut node.kind {
                        NodeKind::Counter(cnt) => {
                            cnt.count = count.get();
                            cnt.fired = fired.get();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub fn create_new_watchpoint(&mut self, wp: Watchpoint) -> Option<NodeId> {
        match self.drag {
            DragState::Idle => {}
            _ => {
                return None;
            }
        }

        let input_id = self.graph.add_node(NodeKind::Condition(wp), Pos2::ZERO);
        let output_id = self
            .graph
            .add_node(NodeKind::Break { lit: false }, Pos2::ZERO);

        self.graph.add_wire(Wire {
            from: Port::new(input_id, 0),
            to: Port::new(output_id, 0),
        });

        self.drag = DragState::CreatingWatchpoint((input_id, output_id));

        Some(input_id)
    }

    pub fn create_new_logic(&mut self, kind: NodeKind) {
        match self.drag {
            DragState::Idle => {}
            _ => {
                return;
            }
        }

        let node_id = self.graph.add_node(
            match kind {
                NodeKind::Condition(_) => {
                    return;
                } // Watchpoint nodes created via create_new_watchpoint
                _ => kind,
            },
            Pos2::ZERO,
        );

        self.drag = DragState::CreatingNode(node_id);
    }

    // ── Coordinate transforms ────────────────────────────────────────────────

    /// Screen → canvas.
    fn to_canvas(&self, origin: Pos2, p: Pos2) -> Pos2 {
        (p - origin.to_vec2() - self.pan) / self.zoom
    }

    /// Canvas → screen.
    fn to_screen(&self, origin: Pos2, p: Pos2) -> Pos2 {
        (p.to_vec2() * self.zoom + self.pan + origin.to_vec2()).to_pos2()
    }

    // ── Main entry point ─────────────────────────────────────────────────────

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        app_state: &app::AppState,
        snem_core: &snemcore::Snemulator,
    ) {
        let mut editing_text = false;

        match self.selection {
            Selection::SingleNode(id) => {
                let node = self.graph.nodes.get_mut(id).unwrap();

                match &mut node.kind {
                    NodeKind::Condition(wp) => {
                        egui::SidePanel::right("condition_editor_panel")
                            .resizable(true)
                            .min_width(250.0)
                            .show_inside(ui, |ui| {
                                ui.heading("Edit Watchpoint");
                                ui.separator();

                                ui.add_enabled_ui(app_state.is_paused, |ui| {
                                    Self::draw_watchpoint_editor(ui, wp, snem_core);
                                })
                            });
                    }

                    NodeKind::Log(lp) => {
                        egui::SidePanel::right("log_editor_panel")
                            .resizable(true)
                            .min_width(250.0)
                            .show_inside(ui, |ui| {
                                ui.heading("Edit Log Point");
                                ui.separator();

                                ui.add_enabled_ui(app_state.is_paused, |ui| {
                                    editing_text = Self::draw_logpoint_editor(ui, lp);
                                });
                            });
                    }

                    NodeKind::Counter(cnt) => {
                        egui::SidePanel::right("counter_editor_panel")
                            .resizable(true)
                            .min_width(250.0)
                            .show_inside(ui, |ui| {
                                ui.heading("Edit Counter");
                                ui.separator();

                                ui.add_enabled_ui(app_state.is_paused, |ui| {
                                    editing_text = Self::draw_counter_editor(ui, cnt);
                                });
                            });
                    }

                    _ => {}
                }
            }
            _ => {}
        }

        self.signals = self.graph.evaluate(snem_core);

        // ── Canvas ───────────────────────────────────────────────────────────
        let (canvas_response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        let origin = canvas_response.rect.min;

        let pointer_screen = canvas_response.hover_pos().unwrap_or(origin);

        // ── Scroll-to-zoom ────────────────────────────────────────────────────
        // Zoom toward the cursor: the canvas point under the cursor must remain
        // the same before and after the zoom change.
        //   cursor_canvas = (pointer_screen - origin - pan) / zoom   [invariant]
        //   => pan_new = pointer_screen - origin - cursor_canvas * zoom_new
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if canvas_response.hovered() && scroll_delta != 0.0 {
            let cursor_canvas = self.to_canvas(origin, pointer_screen);
            self.zoom =
                (self.zoom * (1.0 + scroll_delta * ZOOM_STEP * 0.1)).clamp(ZOOM_MIN, ZOOM_MAX);
            self.pan = pointer_screen - origin - cursor_canvas.to_vec2() * self.zoom;
        }

        // ── Pan ───────────────────────────────────────────────────────────────
        let shift_held = ui.input(|i| i.modifiers.shift);

        if canvas_response.dragged_by(egui::PointerButton::Middle)
            || (canvas_response.dragged_by(egui::PointerButton::Primary) && shift_held)
        {
            self.pan += canvas_response.drag_delta();
        }

        self.pan = self.pan.clamp(
            Vec2::new(-PAN_MAX_X, -PAN_MAX_Y),
            Vec2::new(PAN_MAX_X, PAN_MAX_Y),
        );

        // ── Background grid ───────────────────────────────────────────────────
        self.draw_grid(&painter, canvas_response.rect);

        // ── Node interactions + draw ──────────────────────────────────────────
        self.process_interactions(
            &painter,
            &canvas_response,
            origin,
            shift_held,
            app_state.is_paused,
        );

        // ── Draw wires ────────────────────────────────────────────────────────
        for (idx, wire) in self.graph.wires.iter().enumerate() {
            if let (Some(fn_), Some(tn)) = (
                self.graph.nodes.get(wire.from.node),
                self.graph.nodes.get(wire.to.node),
            ) {
                let from_pos = self.to_screen(origin, fn_.output_port_pos(wire.from.port));
                let to_pos = self.to_screen(origin, tn.input_port_pos(wire.to.port));
                let hot = self.signals.get(&wire.from).copied().unwrap_or(false);
                let selected = matches!(&self.selection, Selection::Wire(i) if *i == idx);
                draw_wire(&painter, from_pos, to_pos, hot, selected);
            }
        }

        // ── Dangling wire preview ─────────────────────────────────────────────
        if let DragState::DraggingWire { from, cursor } = &self.drag {
            if let Some(fn_) = self.graph.nodes.get(from.node) {
                let from_pos = self.to_screen(origin, fn_.output_port_pos(from.port));
                let tip = self.to_screen(origin, *cursor);
                draw_wire(&painter, from_pos, tip, false, false);
            }
        }

        // ── Marquee overlay ───────────────────────────────────────────────────
        if let DragState::DraggingMarquee { anchor, current } = &self.drag {
            let r = Rect::from_two_pos(
                self.to_screen(origin, *anchor),
                self.to_screen(origin, *current),
            );
            painter.rect_filled(r, 2.0, Color32::from_rgba_unmultiplied(80, 140, 255, 30));
            painter.rect_stroke(
                r,
                2.0,
                Stroke::new(1.5, Color32::from_rgb(80, 140, 255)),
                egui::StrokeKind::Middle,
            );
        }

        // ── Delete key ────────────────────────────────────────────────────────
        let try_delete_node =
            ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
        if try_delete_node && !editing_text && app_state.is_paused {
            match &self.selection {
                Selection::SingleNode(id) => {
                    let id = *id;
                    self.graph.remove_node(id);
                    self.selection = Selection::None;
                }
                Selection::MultiNode(set) => {
                    let ids: Vec<NodeId> = set.iter().copied().collect();
                    for id in ids {
                        self.graph.remove_node(id);
                    }
                    self.selection = Selection::None;
                }
                Selection::Wire(idx) => {
                    let idx = *idx;
                    if idx < self.graph.wires.len() {
                        self.graph.wires.remove(idx);
                    }
                    self.selection = Selection::None;
                }
                Selection::None => match self.drag {
                    DragState::CreatingNode(id) => {
                        self.graph.remove_node(id);
                        self.drag = DragState::Idle;
                    }
                    DragState::CreatingWatchpoint((id1, id2)) => {
                        self.graph.remove_node(id1);
                        self.graph.remove_node(id2);
                        self.drag = DragState::Idle;
                    }
                    _ => {}
                },
            }
        }

        // ── Status bar ───────────────────────────────────────────────────────
        let hint = "Scroll to zoom  •  Shift+drag to pan  •  Click port to select its wire";
        painter.text(
            canvas_response.rect.left_bottom() + Vec2::new(8.0, -8.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "{hint}   [zoom: {:.0}%] [pos: {:.0}, {:.0}]",
                self.zoom * 100.0,
                -self.pan.x,
                self.pan.y
            ),
            egui::FontId::proportional(11.0),
            Color32::from_gray(120),
        );
    }

    /// Returns `true` if currently editing text
    fn draw_watchpoint_editor(
        ui: &mut egui::Ui,
        wp: &mut Watchpoint,
        snem_core: &snemcore::Snemulator,
    ) -> bool {
        let mut current_reg_type = wp.val.category();
        let old_reg_type = current_reg_type;
        let was_flag = wp.val.reg_size() == RegSize::Bool;

        Self::draw_category_selector(ui, &mut current_reg_type, 0);

        if current_reg_type != old_reg_type {
            match current_reg_type {
                RegCategory::CpuReg => wp.val = Box::new(CpuReg::A),
                RegCategory::CpuFlag => wp.val = Box::new(CpuFlag::C),
                RegCategory::SysInfo => wp.val = Box::new(SystemVariable::Frame),
                RegCategory::HwReg => wp.val = Box::new(HardwareReg::ApuIo0),
                _ => {}
            };

            let is_numeric = matches!(
                current_reg_type,
                RegCategory::CpuReg | RegCategory::HwReg | RegCategory::SysInfo
            );
            let was_numeric = matches!(
                old_reg_type,
                RegCategory::CpuReg | RegCategory::HwReg | RegCategory::SysInfo
            );

            if is_numeric && !was_numeric {
                wp.cond = WatchpointCond::Equal;
                wp.arg1 = 0;
            }

            if !is_numeric && was_numeric {
                wp.cond = WatchpointCond::Set;
                wp.arg1 = 0;
            }
        }

        wp.kind = current_reg_type;

        let editing = match current_reg_type {
            RegCategory::CpuReg => Self::cpu_reg_wp_edit(ui, wp, snem_core),

            RegCategory::CpuFlag => {
                Self::cpu_flag_wp_edit(ui, wp, snem_core);

                false
            }

            RegCategory::HwReg | RegCategory::HwFlag => Self::hardware_val_wp_edit(ui, wp),

            RegCategory::SysInfo => Self::system_wp_edit(ui, wp, snem_core),

            _ => todo!(),
        };

        if was_flag {
            if wp.val.reg_size() != RegSize::Bool {
                wp.cond = WatchpointCond::Equal;
            }
        } else {
            if wp.val.reg_size() == RegSize::Bool {
                wp.cond = WatchpointCond::Set;
            }
        }

        editing
    }

    /// Returns `true` if text is currently being edited
    fn draw_logpoint_editor(ui: &mut egui::Ui, lp: &mut Logpoint) -> bool {
        let mut editing = false;

        ui.checkbox(&mut lp.message_only, "Message Only");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(monospace_text("Log the values:".to_string()));

            if ui.button("Add New").clicked() {
                lp.regs.push(Box::new(CpuReg::A));
                lp.reg_types.push(RegCategory::CpuReg);
            }
        });

        ui.add_enabled_ui(!lp.message_only, |ui| {
            ui.vertical(|ui| {
                let mut to_remove = None;

                for idx in 0..lp.regs.len() {
                    ui.horizontal(|ui| {
                        if ui.small_button("❌").clicked() {
                            to_remove = Some(idx);
                        }

                        let reg_type = &mut lp.reg_types[idx];
                        let old_reg_type = reg_type.clone();

                        Self::draw_category_selector(ui, reg_type, idx);

                        let reg = &mut lp.regs[idx];

                        if old_reg_type != *reg_type {
                            *reg = match reg_type {
                                RegCategory::CpuReg => Box::new(CpuReg::A),
                                RegCategory::CpuFlag => Box::new(CpuFlag::C),
                                RegCategory::HwReg => Box::new(HardwareReg::ApuIo0),
                                RegCategory::HwFlag => Box::new(HardwareFlag::VBlank),
                                RegCategory::SysInfo => Box::new(SystemVariable::Frame),
                                _ => todo!(),
                            };
                        }

                        match reg_type {
                            RegCategory::CpuReg => {
                                let cpu_reg = reg.as_any_mut().downcast_mut::<CpuReg>().unwrap();

                                Self::draw_cpu_reg_selector(ui, cpu_reg, idx);
                            }
                            RegCategory::CpuFlag => {
                                let cpu_flag = reg.as_any_mut().downcast_mut::<CpuFlag>().unwrap();

                                Self::draw_cpu_flag_selector(ui, cpu_flag, idx);
                            }
                            RegCategory::HwReg | RegCategory::HwFlag => {
                                editing = Self::draw_hardware_reg_selector(
                                    ui,
                                    reg,
                                    reg_type,
                                    &mut lp.hw_reg_search_str,
                                );
                            }
                            RegCategory::SysInfo => {
                                let sys_info =
                                    reg.as_any_mut().downcast_mut::<SystemVariable>().unwrap();

                                Self::draw_system_variable_selector(ui, sys_info, idx);
                            }
                            _ => todo!(),
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    lp.regs.remove(idx);
                    lp.reg_types.remove(idx);
                }
            });
        });

        ui.separator();

        let response = ui.add(
            egui::TextEdit::singleline(&mut lp.msg)
                .hint_text(egui::RichText::new("Message...").italics()),
        );

        if response.has_focus() {
            editing = true;
        }

        editing
    }

    fn draw_counter_editor(ui: &mut egui::Ui, cnt: &mut Counter) -> bool {
        let mut editing = false;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("cnt").code());
            ui.label(monospace_text(format!(": {}", cnt.count)));

            ui.add_space(5.0);

            if ui.button("Reset Count").clicked() {
                cnt.count = 0;
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Increment");

            egui::ComboBox::from_id_salt("counter_inc_mode_sel")
                .selected_text(match cnt.mode {
                    CounterMode::IncOnChange => "on rising edge",
                    CounterMode::IncOnTrue => "on input high",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cnt.mode, CounterMode::IncOnChange, "on rising edge");
                    ui.selectable_value(&mut cnt.mode, CounterMode::IncOnTrue, "on input high");
                })
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(monospace_text("True when cnt".to_string()));

            Self::draw_numeric_cond_selector(ui, &mut cnt.cond, false, 0);

            let response = ui.add(
                egui::TextEdit::singleline(&mut cnt.input_text)
                    .id_salt("cnt_arg")
                    .hint_text(format!("{}", cnt.arg))
                    .desired_width(80.0),
            );

            if response.lost_focus() {
                if let Some(num) = cnt.input_text.trim().parse::<usize>().ok() {
                    cnt.arg = num;
                }
                cnt.input_text.clear();
            }

            editing = response.has_focus();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(monospace_text("Reset when".to_string()));

            egui::ComboBox::from_id_salt("reg_cond_sel")
                .selected_text(if cnt.reset_on_cond { "fired" } else { "cnt is" })
                .width(40.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cnt.reset_on_cond, true, "Fired");
                    ui.selectable_value(&mut cnt.reset_on_cond, false, "Count Reaches");
                });

            if cnt.reset_on_cond {
                cnt.reset = cnt.arg;
            } else {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut cnt.reset_input_text)
                        .id_salt("cnt_reset")
                        .hint_text(format!("{}", cnt.reset))
                        .desired_width(80.0),
                );

                if response.lost_focus() {
                    if let Some(num) = cnt.reset_input_text.trim().parse::<usize>().ok() {
                        cnt.reset = num;
                    }
                    cnt.reset_input_text.clear();
                }

                editing |= response.has_focus();
            }
        });

        ui.separator();

        ui.label(monospace_text(notes::COUNTER_NODE_NOTE.to_string()));

        editing
    }

    /// Returns `true` if currently editing text
    fn cpu_reg_wp_edit(
        ui: &mut egui::Ui,
        wp: &mut Watchpoint,
        snem_core: &snemcore::Snemulator,
    ) -> bool {
        let mut editing = false;

        let reg_size = wp.val.reg_size();
        let old_cond = wp.cond.clone();

        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));

            let reg_ref_mut = wp.val.as_any_mut().downcast_mut::<CpuReg>().unwrap();

            Self::draw_cpu_reg_selector(ui, reg_ref_mut, 0);

            Self::draw_numeric_cond_selector(ui, &mut wp.cond, true, 0);

            editing = Self::draw_numeric_cond_inputs(
                ui,
                &wp.cond,
                reg_size,
                &mut wp.arg1,
                &mut wp.arg2,
                &mut wp.arg1_input_text,
                &mut wp.arg2_input_text,
            );
        });

        if matches!(wp.cond, WatchpointCond::Changed) && wp.cond != old_cond {
            wp.arg1 = wp.val.get_value(snem_core);
        }

        editing
    }

    fn cpu_flag_wp_edit(ui: &mut egui::Ui, wp: &mut Watchpoint, snem_core: &snemcore::Snemulator) {
        let old_cond = wp.cond.clone();

        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));

            let flag_ref_mut = wp.val.as_any_mut().downcast_mut::<CpuFlag>().unwrap();

            Self::draw_cpu_flag_selector(ui, flag_ref_mut, 0);

            ui.label(monospace_text("is".to_string()));

            Self::draw_flag_cond_selector(ui, &mut wp.cond, 0);
        });

        if matches!(wp.cond, WatchpointCond::Changed) && wp.cond != old_cond {
            wp.arg1 = wp.val.get_value(snem_core);
        }

        let flag = wp.val.as_any().downcast_ref::<CpuFlag>().unwrap();

        let note = match flag {
            CpuFlag::C => Some(notes::C_NOTE),
            CpuFlag::Z => Some(notes::Z_NOTE),
            CpuFlag::I => Some(notes::I_NOTE),
            CpuFlag::D => Some(notes::D_NOTE),
            CpuFlag::X => Some(notes::X_NOTE),
            CpuFlag::M => Some(notes::M_NOTE),
            CpuFlag::V => Some(notes::V_NOTE),
            CpuFlag::N => Some(notes::N_NOTE),
            CpuFlag::Stopped => Some(notes::STOPPED_NOTE),
            CpuFlag::NMIPending => Some(notes::NMIPENDING_NOTE),
            CpuFlag::IRQPending => Some(notes::IRQPENDING_NOTE),
            _ => None,
        };

        if let Some(note) = note {
            ui.separator();
            ui.label(monospace_text("NOTES:".to_string()));
            ui.label(monospace_text(note.to_string()));
        }
    }

    fn hardware_val_wp_edit(ui: &mut egui::Ui, wp: &mut Watchpoint) -> bool {
        let mut editing = false;

        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));

            let val_ref_mut = &mut wp.val;

            editing = Self::draw_hardware_reg_selector(
                ui,
                val_ref_mut,
                &mut wp.kind,
                &mut wp.hw_reg_search,
            );

            match wp.kind {
                RegCategory::HwFlag => {
                    Self::draw_flag_cond_selector(ui, &mut wp.cond, 0);
                }
                RegCategory::HwReg => {
                    Self::draw_numeric_cond_selector(ui, &mut wp.cond, true, 0);

                    editing |= Self::draw_numeric_cond_inputs(
                        ui,
                        &wp.cond,
                        wp.val.reg_size(),
                        &mut wp.arg1,
                        &mut wp.arg2,
                        &mut wp.arg1_input_text,
                        &mut wp.arg2_input_text,
                    );
                }
                _ => {}
            }
        });

        editing
    }

    fn system_wp_edit(
        ui: &mut egui::Ui,
        wp: &mut Watchpoint,
        snem_core: &snemcore::Snemulator,
    ) -> bool {
        let mut editing = false;

        let old_cond = wp.cond.clone();

        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));

            let variable_ref_mut = wp
                .val
                .as_any_mut()
                .downcast_mut::<SystemVariable>()
                .unwrap();

            Self::draw_system_variable_selector(ui, variable_ref_mut, 0);

            Self::draw_numeric_cond_selector(ui, &mut wp.cond, false, 0);

            editing = Self::draw_numeric_cond_inputs(
                ui,
                &wp.cond,
                RegSize::Num,
                &mut wp.arg1,
                &mut wp.arg2,
                &mut wp.arg1_input_text,
                &mut wp.arg2_input_text,
            );
        });

        let variable = wp.val.as_any().downcast_ref::<SystemVariable>().unwrap();

        match variable {
            SystemVariable::Dot => {
                ui.separator();

                ui.label(monospace_text("NOTES:".to_string()));
                ui.label(monospace_text(notes::DOT_NOTE.to_string()));
            }
            SystemVariable::Scanline => {
                ui.separator();

                ui.label(monospace_text("NOTES: (NTSC)".to_string()));
                ui.label(monospace_text(notes::SCANLINE_NOTE.to_string()));
            }
            _ => {}
        }

        if matches!(wp.cond, WatchpointCond::Changed) && wp.cond != old_cond {
            wp.arg1 = wp.val.get_value(snem_core);
        }

        editing
    }

    fn draw_category_selector(ui: &mut egui::Ui, category: &mut RegCategory, egui_id: usize) {
        ui.horizontal(|ui| {
            ui.label("Target:");

            egui::ComboBox::from_id_salt(format!("target_type_sel_{}", egui_id))
                .selected_text(category.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(category, RegCategory::CpuReg, RegCategory::CpuReg.label());
                    ui.selectable_value(
                        category,
                        RegCategory::CpuFlag,
                        RegCategory::CpuFlag.label(),
                    );
                    ui.selectable_value(category, RegCategory::Ram, RegCategory::Ram.label());
                    ui.selectable_value(category, RegCategory::Vram, RegCategory::Vram.label());
                    ui.selectable_value(category, RegCategory::HwReg, RegCategory::HwReg.label());
                    ui.selectable_value(
                        category,
                        RegCategory::SysInfo,
                        RegCategory::SysInfo.label(),
                    );
                })
        });

        ui.separator();
    }

    fn draw_cpu_reg_selector(ui: &mut egui::Ui, reg: &mut CpuReg, egui_id: usize) {
        egui::ComboBox::from_id_salt(format!("cpu_reg_sel_{}", egui_id))
            .width(20.0)
            .selected_text(reg.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(reg, CpuReg::DB, "DB (Data Bank)");
                ui.selectable_value(reg, CpuReg::PB, "PB (Program Bank)");
                ui.selectable_value(reg, CpuReg::P, "P (Processor Status)");
                ui.selectable_value(reg, CpuReg::A, "A (Accumulator)");
                ui.selectable_value(reg, CpuReg::X, "X (X Index)");
                ui.selectable_value(reg, CpuReg::Y, "Y (Y Index)");
                ui.selectable_value(reg, CpuReg::DP, "DP (Direct Page)");
                ui.selectable_value(reg, CpuReg::PC, "PC (Program Counter)");
                ui.selectable_value(reg, CpuReg::SP, "SP (Stack Pointer)");
            });
    }

    fn draw_cpu_flag_selector(ui: &mut egui::Ui, flag: &mut CpuFlag, egui_id: usize) {
        egui::ComboBox::from_id_salt(format!("flag_sel_{}", egui_id))
            .width(20.0)
            .selected_text(flag.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(flag, CpuFlag::C, "C (Carry)");
                ui.selectable_value(flag, CpuFlag::Z, "Z (Zero)");
                ui.selectable_value(flag, CpuFlag::I, "I (Interrupt)");
                ui.selectable_value(flag, CpuFlag::D, "D (Decimal)");
                ui.selectable_value(flag, CpuFlag::X, "X (Idx. Size)");
                ui.selectable_value(flag, CpuFlag::M, "M (Acc. Size)");
                ui.selectable_value(flag, CpuFlag::V, "V (Overflow)");
                ui.selectable_value(flag, CpuFlag::N, "N (Negative)");
                ui.selectable_value(flag, CpuFlag::Stopped, "Stopped");
                ui.selectable_value(flag, CpuFlag::Halted, "Halted");
                ui.selectable_value(flag, CpuFlag::Waiting, "Waiting");
                ui.selectable_value(flag, CpuFlag::NMIPending, "NMI Pending");
                ui.selectable_value(flag, CpuFlag::IRQPending, "IRQ Pending");
            });
    }

    /// Returns `true` if the user is editing the search bar.
    fn draw_hardware_reg_selector(
        ui: &mut egui::Ui,
        reg_or_flag: &mut Box<dyn WatchpointValue>,
        val_category: &mut RegCategory,
        search_str: &mut String,
    ) -> bool {
        let mut editing = false;

        ui.horizontal(|ui| {
            ui.set_width(80.0);

            let response = ui.add(
                egui_autocomplete::AutoCompleteTextEdit::new(search_str, &HWVAL_NAMES)
                    .max_suggestions(10)
                    .popup_on_focus(true),
            );

            if response.gained_focus() {
                search_str.clear()
            }

            let submitted = response.lost_focus();
            editing = response.has_focus();

            let old_category = val_category.clone();

            if submitted {
                if let Some(new_reg) = search_str.trim().parse::<HardwareReg>().ok() {
                    *val_category = RegCategory::HwReg;

                    if old_category != RegCategory::HwReg {
                        *reg_or_flag = Box::new(new_reg);
                    } else {
                        let reg_ref_mut = reg_or_flag
                            .as_any_mut()
                            .downcast_mut::<HardwareReg>()
                            .unwrap();

                        *reg_ref_mut = new_reg;
                    }

                    search_str.clear();
                } else if let Some(new_flag) = search_str.trim().parse::<HardwareFlag>().ok() {
                    *val_category = RegCategory::HwFlag;

                    if old_category != RegCategory::HwFlag {
                        *reg_or_flag = Box::new(new_flag);
                    } else {
                        let flag_ref_mut = reg_or_flag
                            .as_any_mut()
                            .downcast_mut::<HardwareFlag>()
                            .unwrap();

                        *flag_ref_mut = new_flag;
                    }

                    search_str.clear();
                }
            }

            if response.lost_focus() {
                *search_str = reg_or_flag.label();
            }
        });

        editing
    }

    fn draw_system_variable_selector(
        ui: &mut egui::Ui,
        variable: &mut SystemVariable,
        egui_id: usize,
    ) {
        egui::ComboBox::from_id_salt(format!("sys_var_sel_{}", egui_id))
            .width(20.0)
            .selected_text(variable.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(variable, SystemVariable::Frame, "Frame");
                ui.selectable_value(variable, SystemVariable::Dot, "Dot");
                ui.selectable_value(variable, SystemVariable::Scanline, "Scanline");
            });
    }

    fn draw_numeric_cond_selector(
        ui: &mut egui::Ui,
        cond: &mut WatchpointCond,
        allow_bitwise: bool,
        egui_id: usize,
    ) {
        egui::ComboBox::from_id_salt(format!("num_cond_sel_{}", egui_id))
            .width(20.0)
            .selected_text(cond.dropdown_label())
            .show_ui(ui, |ui| {
                ui.selectable_value(cond, WatchpointCond::Equal, "== (Equal)");
                ui.selectable_value(cond, WatchpointCond::NotEqual, "!= (Not Equal)");
                ui.selectable_value(cond, WatchpointCond::GreaterThan, "> (Greater Than)");
                ui.selectable_value(cond, WatchpointCond::LessThan, "< (Less Than)");

                if allow_bitwise {
                    ui.selectable_value(cond, WatchpointCond::AndEqual, "& (Bitwise AND)");
                    ui.selectable_value(cond, WatchpointCond::OrEqual, "| (Bitwise OR)");
                }

                ui.selectable_value(cond, WatchpointCond::Changed, "Changed");
            });
    }

    fn draw_flag_cond_selector(ui: &mut egui::Ui, cond: &mut WatchpointCond, egui_id: usize) {
        egui::ComboBox::from_id_salt(format!("flag_cond_{}", egui_id))
            .width(20.0)
            .selected_text(cond.dropdown_label())
            .show_ui(ui, |ui| {
                ui.selectable_value(cond, WatchpointCond::Set, "Set");
                ui.selectable_value(cond, WatchpointCond::Clear, "Clear");
                ui.selectable_value(cond, WatchpointCond::Changed, "Changed");
            });
    }

    /// Returns `true` if the user is currently editing condition input text.
    fn draw_numeric_cond_inputs(
        ui: &mut egui::Ui,
        cond_type: &WatchpointCond,
        reg_size: RegSize,
        arg1: &mut usize,
        arg2: &mut usize,
        cond_edit_arg1_text: &mut String,
        cond_edit_arg2_text: &mut String,
    ) -> bool {
        let mut editing = false;

        let desired_width = match reg_size {
            RegSize::Word => 40.0,
            _ => 20.0,
        };
        let arg1_hint_text = match reg_size {
            RegSize::Byte => format!("{:02x}", arg1),
            RegSize::Word => format!("{:04x}", arg1),
            RegSize::Num => format!("{}", arg1),
            _ => "".to_string(),
        };
        let arg2_hint_text = match reg_size {
            RegSize::Byte => format!("{:02x}", arg2),
            RegSize::Word => format!("{:04x}", arg2),
            RegSize::Num => format!("{}", arg2),
            _ => "".to_string(),
        };

        ui.horizontal(|ui| match cond_type {
            WatchpointCond::Equal
            | WatchpointCond::NotEqual
            | WatchpointCond::GreaterThan
            | WatchpointCond::LessThan => {
                editing = Self::reg_input_box(
                    ui,
                    desired_width,
                    arg1_hint_text,
                    reg_size,
                    arg1,
                    cond_edit_arg1_text,
                    0,
                );
            }
            WatchpointCond::OrEqual | WatchpointCond::AndEqual => {
                ui.horizontal(|ui| {
                    editing = Self::reg_input_box(
                        ui,
                        desired_width,
                        arg1_hint_text,
                        reg_size,
                        arg1,
                        cond_edit_arg1_text,
                        0,
                    );

                    ui.label(monospace_text("==".to_string()));

                    editing |= Self::reg_input_box(
                        ui,
                        desired_width,
                        arg2_hint_text,
                        reg_size,
                        arg2,
                        cond_edit_arg2_text,
                        1,
                    );
                });
            }
            WatchpointCond::Changed | WatchpointCond::Set | WatchpointCond::Clear => {}
        });

        editing
    }

    /// Returns `true` if the input box is currently being edited.
    fn reg_input_box(
        ui: &mut egui::Ui,
        desired_width: f32,
        hint_text: String,
        reg_size: RegSize,
        num: &mut usize,
        cond_edit_text: &mut String,
        egui_id: usize,
    ) -> bool {
        let response = ui.add(
            egui::TextEdit::singleline(cond_edit_text)
                .id_salt(format!("reg_input_{}", egui_id))
                .desired_width(desired_width)
                .hint_text(hint_text),
        );

        let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if submitted {
            match reg_size {
                RegSize::Byte => {
                    if let Ok(val) = u8::from_str_radix(cond_edit_text.trim(), 16) {
                        *num = val as usize;
                    }
                }
                RegSize::Word => {
                    if let Ok(val) = u16::from_str_radix(cond_edit_text.trim(), 16) {
                        *num = val as usize;
                    }
                }
                RegSize::Num => {
                    if let Ok(val) = cond_edit_text.trim().parse::<usize>() {
                        *num = val;
                    }
                }
                _ => {}
            }
            cond_edit_text.clear()
        }

        response.has_focus()
    }

    // ── Interaction processing ────────────────────────────────────────────────

    /// Processes interactions with the canvas (dragging, selecting, etc.) and returns `true` if the graph was modified.
    fn process_interactions(
        &mut self,
        painter: &egui::Painter,
        response: &egui::Response,
        origin: Pos2,
        shift_held: bool,
        emulation_paused: bool,
    ) {
        let pointer_screen = response.hover_pos().unwrap_or(origin);
        let pointer_canvas = self.to_canvas(origin, pointer_screen);

        let ids: Vec<NodeId> = self.graph.nodes.keys().collect();

        // ── Tick active drag ──────────────────────────────────────────────────
        match &mut self.drag {
            DragState::DraggingNodes(offsets) => {
                let offsets_snap: Vec<(NodeId, Vec2)> = offsets.clone();
                if response.drag_stopped() {
                    self.drag = DragState::Idle;
                } else {
                    for (id, off) in &offsets_snap {
                        if let Some(node) = self.graph.nodes.get_mut(*id) {
                            node.pos = pointer_canvas - *off;
                        }
                    }
                }
            }

            DragState::DraggingWire { from, cursor } => {
                *cursor = pointer_canvas;
                if response.drag_stopped() {
                    let from = *from;
                    let mut snapped: Option<Port> = None;
                    'snap: for &nid in &ids {
                        if let Some(node) = self.graph.nodes.get(nid) {
                            for i in 0..node.kind.input_count() {
                                // let pp = self.to_screen(origin, node.input_port_pos(i));
                                // if pp.distance(pointer_screen) < PORT_HIT_RADIUS {
                                let pp = node.input_port_pos(i);
                                if pp.distance(pointer_canvas) < PORT_HIT_RADIUS {
                                    snapped = Some(Port::new(nid, i));
                                    break 'snap;
                                }
                            }
                        }
                    }
                    if let Some(to) = snapped {
                        if to.node != from.node {
                            self.graph.add_wire(Wire { from, to });
                        }
                    }
                    self.drag = DragState::Idle;
                }
            }

            DragState::DraggingMarquee { anchor, current } => {
                *current = pointer_canvas;
                if response.drag_stopped() {
                    let marquee = Rect::from_two_pos(*anchor, *current);
                    let selected: HashSet<NodeId> = ids
                        .iter()
                        .filter(|&&id| {
                            self.graph
                                .nodes
                                .get(id)
                                .map(|n| marquee.intersects(n.rect()))
                                .unwrap_or(false)
                        })
                        .copied()
                        .collect();
                    self.selection = match selected.len() {
                        0 => Selection::None,
                        1 => Selection::SingleNode(*selected.iter().next().unwrap()),
                        _ => Selection::MultiNode(selected),
                    };
                    self.drag = DragState::Idle;
                }
            }

            DragState::CreatingNode(node_id) => {
                if response.clicked() {
                    self.drag = DragState::Idle;
                } else {
                    self.graph.nodes.get_mut(*node_id).unwrap().pos = pointer_canvas;
                }
            }

            DragState::CreatingWatchpoint(nodes) => {
                if response.clicked() {
                    self.drag = DragState::Idle;
                } else {
                    let offset = Vec2::new(Node::WIDTH * 2.5, 0.0);

                    let input_node = self.graph.nodes.get_mut(nodes.0).unwrap();
                    input_node.pos = pointer_canvas;

                    let output_node = self.graph.nodes.get_mut(nodes.1).unwrap();
                    output_node.pos = pointer_canvas + offset;
                }
            }

            DragState::Idle => {}
        }

        if response.clicked_by(egui::PointerButton::Primary) {
            self.selection = Selection::None;
        }

        // ── Per-node rendering + hit testing ─────────────────────────────────
        for &id in &ids {
            let Some(node) = self.graph.nodes.get(id) else {
                continue;
            };

            let node_rect_screen = Rect::from_min_size(
                self.to_screen(origin, node.pos),
                egui::vec2(Node::WIDTH * self.zoom, Node::HEIGHT * self.zoom),
            );

            // ── Drag start (in Select mode only) ─────────────────────────────
            if matches!(self.drag, DragState::Idle)
                && response.drag_started_by(egui::PointerButton::Primary)
            {
                let node = self.graph.nodes.get(id).unwrap();

                // Check output ports first — they take priority over node body.
                let mut started_wire = false;
                if emulation_paused {
                    for o in 0..node.kind.output_count() {
                        let pp = self.to_screen(origin, node.output_port_pos(o));
                        if pp.distance(pointer_screen) < PORT_HIT_RADIUS {
                            self.drag = DragState::DraggingWire {
                                from: Port::new(id, o),
                                cursor: pointer_canvas,
                            };
                            started_wire = true;
                            break;
                        }
                    }
                }

                // Node body → move selected nodes.
                if !started_wire && node_rect_screen.contains(pointer_screen) {
                    let drag_ids = if !self.selection.contains_node(id) {
                        vec![id]
                    } else {
                        self.selection.node_ids()
                    };
                    let offsets: Vec<(NodeId, Vec2)> = drag_ids
                        .into_iter()
                        .filter_map(|nid| {
                            self.graph
                                .nodes
                                .get(nid)
                                .map(|n| (nid, pointer_canvas - n.pos))
                        })
                        .collect();
                    self.drag = DragState::DraggingNodes(offsets);
                }
            }

            // ── Click on input/output port → select its wire ─────────────────────────
            // (Only when we're not in the middle of a drag.)
            if matches!(self.drag, DragState::Idle)
                && response.clicked_by(egui::PointerButton::Primary)
                && !shift_held
            {
                let node = self.graph.nodes.get(id).unwrap();
                for i in 0..node.kind.input_count() {
                    let pp = self.to_screen(origin, node.input_port_pos(i));
                    if pp.distance(pointer_screen) < PORT_HIT_RADIUS {
                        let target = Port::new(id, i);
                        if let Some(wire_idx) = self.graph.wires.iter().position(|w| w.to == target)
                        {
                            // Toggle: clicking the same wire deselects it.
                            self.selection = match &self.selection {
                                Selection::Wire(old) if *old == wire_idx => Selection::None,
                                _ => Selection::Wire(wire_idx),
                            };
                        }
                        // Either way, stop here — don't also toggle the switch below.
                        break;
                    }
                }

                for i in 0..node.kind.output_count() {
                    let pp = self.to_screen(origin, node.output_port_pos(i));
                    if pp.distance(pointer_screen) < PORT_HIT_RADIUS {
                        let target = Port::new(id, i);
                        if let Some(wire_idx) =
                            self.graph.wires.iter().position(|w| w.from == target)
                        {
                            // Toggle: clicking the same wire deselects it.
                            self.selection = match &self.selection {
                                Selection::Wire(old) if *old == wire_idx => Selection::None,
                                _ => Selection::Wire(wire_idx),
                            };
                        }
                        // Either way, stop here — don't also toggle the switch below.
                        break;
                    }
                }
            }

            // Select node on primary click
            if response.clicked_by(egui::PointerButton::Primary)
                && node_rect_screen.contains(pointer_screen)
            {
                // Make sure we're not on an input port (already handled above).
                let node = self.graph.nodes.get(id).unwrap();
                let on_input_port = (0..node.kind.input_count()).any(|i| {
                    self.to_screen(origin, node.input_port_pos(i))
                        .distance(pointer_screen)
                        < PORT_HIT_RADIUS
                });
                let on_output_port = (0..node.kind.output_count()).any(|i| {
                    self.to_screen(origin, node.output_port_pos(i))
                        .distance(pointer_screen)
                        < PORT_HIT_RADIUS
                });
                if !on_input_port && !on_output_port {
                    self.selection = Selection::SingleNode(id);
                }
            }

            // ── Draw the node ─────────────────────────────────────────────────
            let node = self.graph.nodes.get(id).unwrap();
            self.draw_node(painter, id, node, node_rect_screen, origin);
        }

        // ── Marquee drag start: only when no node was hit ─────────────────────
        if matches!(self.drag, DragState::Idle)
            && response.drag_started_by(egui::PointerButton::Primary)
        {
            let hit_any = ids.iter().any(|&id| {
                self.graph
                    .nodes
                    .get(id)
                    .map(|n| {
                        Rect::from_min_size(
                            self.to_screen(origin, n.pos),
                            egui::vec2(Node::WIDTH * self.zoom, Node::HEIGHT * self.zoom),
                        )
                        .contains(pointer_screen)
                    })
                    .unwrap_or(false)
            });
            if !hit_any {
                // self.selection = Selection::None;
                self.drag = DragState::DraggingMarquee {
                    anchor: pointer_canvas,
                    current: pointer_canvas,
                };
            }
        }
    }

    // ── Node rendering ────────────────────────────────────────────────────────

    fn draw_node(
        &self,
        painter: &egui::Painter,
        id: NodeId,
        node: &Node,
        rect: Rect,
        origin: Pos2,
    ) {
        let base_color = node.kind.color();
        let is_selected = self.selection.contains_node(id);

        let fill = if let NodeKind::Break { lit: true } = node.kind {
            Color32::from_rgb(255, 100, 80)
        } else {
            base_color.linear_multiply(0.5)
        };

        // Drop shadow.
        painter.rect_filled(
            rect.translate(Vec2::splat(3.0)),
            6.0,
            Color32::from_black_alpha(60),
        );
        // Body.
        painter.rect_filled(rect, 6.0, fill);
        // Border — white when selected.
        let border_color = if is_selected {
            Color32::WHITE
        } else {
            base_color
        };
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(if is_selected { 2.5 } else { 1.5 }, border_color),
            egui::StrokeKind::Middle,
        );

        let font_size = 13.0 * self.zoom;
        let label = match &node.kind {
            NodeKind::Condition(wp_kind) => wp_kind.label(),
            NodeKind::Log(log_kind) => log_kind.label(),
            other => other.label(),
        };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::monospace(font_size),
            Color32::WHITE,
        );

        // Port radius scales sub-linearly with zoom so ports don't dominate.
        let port_r = PORT_RADIUS * self.zoom.sqrt();

        // Input ports.
        for i in 0..node.kind.input_count() {
            let pos = self.to_screen(origin, node.input_port_pos(i));
            let target = Port::new(id, i);
            let wire_idx = self.graph.wires.iter().position(|w| w.to == target);

            let signal = wire_idx
                .and_then(|wi| self.signals.get(&self.graph.wires[wi].from).copied())
                .unwrap_or(false);
            let wire_selected = wire_idx
                .map(|wi| matches!(&self.selection, Selection::Wire(si) if *si == wi))
                .unwrap_or(false);

            let port_color = if wire_selected {
                Color32::from_rgb(255, 220, 50) // yellow = selected wire
            } else if signal {
                Color32::from_rgb(100, 255, 100)
            } else {
                Color32::from_gray(140)
            };
            painter.circle_filled(pos, port_r, port_color);
            painter.circle_stroke(pos, port_r, Stroke::new(1.5, Color32::WHITE));
        }

        // Output ports.
        for o in 0..node.kind.output_count() {
            let pos = self.to_screen(origin, node.output_port_pos(o));
            let signal = self
                .signals
                .get(&Port::new(id, o))
                .copied()
                .unwrap_or(false);
            let port_color = if signal {
                Color32::from_rgb(100, 255, 100)
            } else {
                Color32::from_gray(180)
            };
            painter.circle_filled(pos, port_r, port_color);
            painter.circle_stroke(pos, port_r, Stroke::new(1.5, Color32::WHITE));
        }
    }

    // ── Grid ─────────────────────────────────────────────────────────────────

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        painter.rect_filled(rect, 0.0, Color32::from_gray(18));

        // Grid spacing follows zoom so it never crowds.
        let spacing = 32.0 * self.zoom;
        if spacing < 6.0 {
            return;
        }

        let offset_x = self.pan.x.rem_euclid(spacing);
        let offset_y = self.pan.y.rem_euclid(spacing);

        let mut x = rect.left() + offset_x;
        while x < rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(30)),
            );
            x += spacing;
        }
        let mut y = rect.top() + offset_y;
        while y < rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(30)),
            );
            y += spacing;
        }
    }
}

// ── Wire drawing ──────────────────────────────────────────────────────────────

/// Cubic-Bezier wire. `selected` draws a yellow halo underneath the wire.
fn draw_wire(painter: &egui::Painter, from: Pos2, to: Pos2, hot: bool, selected: bool) {
    let color = if hot {
        Color32::from_rgb(80, 240, 100)
    } else {
        Color32::from_gray(160)
    };

    let dx = (to.x - from.x).abs().max(60.0);
    let ctrl1 = from + Vec2::new(dx * 0.5, 0.0);
    let ctrl2 = to - Vec2::new(dx * 0.5, 0.0);
    let points = [from, ctrl1, ctrl2, to];

    // Selection halo drawn first (behind).
    if selected {
        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
            points,
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: egui::Stroke::new(WIRE_THICKNESS + 4.0, Color32::from_rgb(255, 220, 50)).into(),
        }));
    }

    painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
        points,
        closed: false,
        fill: Color32::TRANSPARENT,
        stroke: egui::Stroke::new(WIRE_THICKNESS, color).into(),
    }));

    painter.circle_filled(from, 3.5, color);
}
