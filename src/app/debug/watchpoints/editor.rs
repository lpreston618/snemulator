use crate::app;
use crate::core::scpu;
use crate::core::snemcore;
use crate::app::utils::monospace_text;
use crate::app::debug::watchpoints::notes;
use crate::app::debug::watchpoints::types::*;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

// ── Constants ────────────────────────────────────────────────────────────────

const PORT_RADIUS: f32 = 6.0;
const WIRE_THICKNESS: f32 = 2.5;
/// Radius (in screen pixels) within which a port registers a hit.
const PORT_HIT_RADIUS: f32 = 14.0;
const ZOOM_MIN: f32 = 0.50;
const ZOOM_MAX: f32 = 2.00;
const ZOOM_STEP: f32 = 0.10;
const PAN_MAX_X: f32 = 1000.0;
const PAN_MAX_Y: f32 = 1000.0;

#[derive(PartialEq, Clone, Copy)]
enum RegCategory { CpuReg, Flag, Ram, Vram, HwReg, SysInfo }
#[derive(PartialEq, Clone, Copy)]
enum RegCondType { Eq, NEq, Gt, GtEq, AndEq, OrEq, Changed }

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
    CreatingWatchpoint( (NodeId, NodeId) ),
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
    cond_edit_arg1_text: String,
    cond_edit_arg2_text: String,
    editing_text: bool,
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
            cond_edit_arg1_text: String::new(),
            cond_edit_arg2_text: String::new(),
            editing_text: false,
        }
    }
    
    pub fn create_new_watchpoint(&mut self, kind: WatchpointKind) -> Option<NodeId> {
        match self.drag {
            DragState::Idle => {}
            _ => { return None; }
        }
        
        let input_id = self.graph.add_node(NodeKind::Condition(kind), Pos2::ZERO);
        let output_id = self.graph.add_node(NodeKind::Break { lit: false }, Pos2::ZERO);
        
        self.graph.add_wire(Wire {
            from: Port::new(input_id, 0),
            to:   Port::new(output_id, 0),
        });
        
        self.drag = DragState::CreatingWatchpoint((input_id, output_id));
    
        Some(input_id)
    }
    
    pub fn create_new_logic(&mut self, kind: NodeKind) {
        match self.drag {
            DragState::Idle => {},
            _ => { return; }
        }
        
        let node_id = self.graph.add_node(match kind {
            NodeKind::Condition(_) => { return; }, // Watchpoint nodes created via create_new_watchpoint
            _ => kind,
        }, Pos2::ZERO);
        
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

    pub fn show(&mut self, ui: &mut egui::Ui, app_state: &app::AppState, snem_core: &snemcore::Snemulator) {
        
        self.editing_text = false;
        if app_state.is_paused {
            match self.selection {
                Selection::SingleNode(id) => {
                    let node = self.graph.nodes.get(id).unwrap();
                    
                    if matches!(node.kind, NodeKind::Condition(_)) {
                        egui::SidePanel::right("condition_editor_panel")
                            .resizable(true)
                            .min_width(250.0)
                            .show_inside(ui, |ui| {
                                ui.heading("Edit Condition");
                                ui.separator();
                                
                                self.draw_condition_editor(ui, id, snem_core);
                            });
                    }
                    
                    let node = self.graph.nodes.get(id).unwrap();
                    
                    if matches!(node.kind, NodeKind::Log(_)) {
                        egui::SidePanel::right("log_editor_panel")
                            .resizable(true)
                            .min_width(250.0)
                            .show_inside(ui, |ui| {
                                ui.heading("Edit Log Point");
                                ui.separator();
                                
                                self.draw_log_editor(ui, id);
                            });
                    }
                    
                }
                _ => {}
            }
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
            self.zoom = (self.zoom * (1.0 + scroll_delta * ZOOM_STEP * 0.1))
                .clamp(ZOOM_MIN, ZOOM_MAX);
            self.pan = pointer_screen - origin - cursor_canvas.to_vec2() * self.zoom;
        }

        // ── Pan ───────────────────────────────────────────────────────────────
        let shift_held = ui.input(|i| i.modifiers.shift);
        
        if canvas_response.dragged_by(egui::PointerButton::Middle) || (canvas_response.dragged_by(egui::PointerButton::Primary) && shift_held) {
            self.pan += canvas_response.drag_delta();
        }
        
        self.pan = self.pan.clamp(
            Vec2::new(-PAN_MAX_X, -PAN_MAX_Y),
            Vec2::new(PAN_MAX_X, PAN_MAX_Y),
        );

        // ── Background grid ───────────────────────────────────────────────────
        self.draw_grid(&painter, canvas_response.rect);

        // ── Node interactions + draw ──────────────────────────────────────────
        self.process_interactions(&painter, &canvas_response, origin, shift_held);

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
        if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) && !self.editing_text {
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
                Selection::None => {
                    match self.drag {
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
                    }
                }
            }
        }

        // ── Status bar ───────────────────────────────────────────────────────
        let hint = "Scroll to zoom  •  Shift+drag to pan  •  Click port to select its wire";
        painter.text(
            canvas_response.rect.left_bottom() + Vec2::new(8.0, -8.0),
            egui::Align2::LEFT_BOTTOM,
            format!("{hint}   [zoom: {:.0}%] [pos: {:.0}, {:.0}]", self.zoom * 100.0, -self.pan.x, self.pan.y),
            egui::FontId::proportional(11.0),
            Color32::from_gray(120),
        );
    }
    
    fn draw_category_selector(ui: &mut egui::Ui, category: &mut RegCategory) {
        ui.horizontal(|ui| {
            ui.label("Target:");
            
            egui::ComboBox::from_id_salt("target_type_sel")
                .selected_text(
                    match category {
                        RegCategory::CpuReg => "CPU Register",
                        RegCategory::Flag => "Hardware/CPU Flag",
                        RegCategory::Ram => "RAM",
                        RegCategory::Vram => "VRAM",
                        RegCategory::HwReg => "Hardware Register",
                        RegCategory::SysInfo => "System Info",
                    })
                .show_ui(ui, |ui| {
                    ui.selectable_value(category, RegCategory::CpuReg, "CPU Register");
                    ui.selectable_value(category, RegCategory::Flag, "Hardware/CPU Flag");
                    ui.selectable_value(category, RegCategory::Ram, "RAM");
                    ui.selectable_value(category, RegCategory::Vram, "VRAM");
                    ui.selectable_value(category, RegCategory::HwReg, "Hardware Register");
                    ui.selectable_value(category, RegCategory::SysInfo, "System Info");
                })
        });

        ui.separator();
    }
    
    fn draw_cpu_reg_selector(ui: &mut egui::Ui, reg: &mut CpuReg) {
        egui::ComboBox::from_id_salt("reg_sel").width(20.0)
            .selected_text(match reg {
                CpuReg::DB => "DB",
                CpuReg::PB => "PB",
                CpuReg::P  => "P",
                CpuReg::A => "A",
                CpuReg::X => "X",
                CpuReg::Y => "Y",
                CpuReg::DP => "DP",
                CpuReg::PC => "PC",
                CpuReg::SP => "SP",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(reg, CpuReg::DB, "DB (Data Bank)"      );
                ui.selectable_value(reg, CpuReg::PB, "PB (Program Bank)"   );
                ui.selectable_value(reg, CpuReg::P,  "P (Processor Status)");
                ui.selectable_value(reg, CpuReg::A,  "A (Accumulator)"     );
                ui.selectable_value(reg, CpuReg::X,  "X (X Index)"         );
                ui.selectable_value(reg, CpuReg::Y,  "Y (Y Index)"         );
                ui.selectable_value(reg, CpuReg::DP, "DP (Direct Page)"    );
                ui.selectable_value(reg, CpuReg::PC, "PC (Program Counter)");
                ui.selectable_value(reg, CpuReg::SP, "SP (Stack Pointer)"  );
            });
    }
    
    fn draw_cpu_flag_selector(ui: &mut egui::Ui, flag: &mut CpuFlag) {
        egui::ComboBox::from_id_salt("flag_sel").width(20.0)
            .selected_text(match flag {
                CpuFlag::C => "C",
                CpuFlag::Z => "Z",
                CpuFlag::I => "I",
                CpuFlag::D => "D",
                CpuFlag::X => "X",
                CpuFlag::M => "M",
                CpuFlag::V => "V",
                CpuFlag::N => "N",
                CpuFlag::Stopped => "Stopped",
                CpuFlag::Halted => "Halted",
                CpuFlag::Waiting => "Waiting",
                CpuFlag::NMIPending => "NMI Pending",
                CpuFlag::IRQPending => "IRQ Pending",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(flag, CpuFlag::C,          "C (Carry)"    );
                ui.selectable_value(flag, CpuFlag::Z,          "Z (Zero)"     );
                ui.selectable_value(flag, CpuFlag::I,          "I (Interrupt)");
                ui.selectable_value(flag, CpuFlag::D,          "D (Decimal)"  );
                ui.selectable_value(flag, CpuFlag::X,          "X (Idx. Size)");
                ui.selectable_value(flag, CpuFlag::M,          "M (Acc. Size)");
                ui.selectable_value(flag, CpuFlag::V,          "V (Overflow)" );
                ui.selectable_value(flag, CpuFlag::N,          "N (Negative)" );
                ui.selectable_value(flag, CpuFlag::Stopped,    "Stopped"      );
                ui.selectable_value(flag, CpuFlag::Halted,     "Halted"       );
                ui.selectable_value(flag, CpuFlag::Waiting,    "Waiting"      );
                ui.selectable_value(flag, CpuFlag::NMIPending, "NMI Pending"   );
                ui.selectable_value(flag, CpuFlag::IRQPending, "IRQ Pending"   );
            });
    }
    
    fn draw_system_variable_selector(ui: &mut egui::Ui, variable: &mut SystemVariable) {
        egui::ComboBox::from_id_salt("sys_var_sel").width(20.0)
            .selected_text(match variable {
                SystemVariable::Frame => "Frame",
                SystemVariable::Dot => "Dot",
                SystemVariable::Scanline => "Scanline",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(variable, SystemVariable::Frame, "Frame");
                ui.selectable_value(variable, SystemVariable::Dot, "Dot");
                ui.selectable_value(variable, SystemVariable::Scanline, "Scanline");
            });
    }

    pub fn draw_condition_editor(&mut self, ui: &mut egui::Ui, id: NodeId, snem_core: &snemcore::Snemulator) {
        let node = match self.graph.nodes.get_mut(id) {
            Some(n) => n,
            None => return,
        };

        let NodeKind::Condition(wp_kind) = &mut node.kind else {
            return;
        };

        let mut current_cat = match wp_kind {
            WatchpointKind::CpuReg { .. } => RegCategory::CpuReg,
            WatchpointKind::CpuFlag { .. } => RegCategory::Flag,
            WatchpointKind::System { .. } => RegCategory::SysInfo,
        };
        let old_cat = current_cat;

        Self::draw_category_selector(ui, &mut current_cat);

        if old_cat != current_cat {
            match current_cat {
                RegCategory::CpuReg => *wp_kind = WatchpointKind::CpuReg { reg: CpuReg::A, cond: WatchpointCond::Equal(0) },
                RegCategory::Flag => *wp_kind = WatchpointKind::CpuFlag { flag: CpuFlag::C, cond: WatchpointCondFlag::Set },
                RegCategory::SysInfo => *wp_kind = WatchpointKind::System { variable: SystemVariable::Frame, cond: WatchpointCond::Equal(0) },
                // Category::Ram => WatchpointKind::WPRam {  },
                // Category::Vram => WatchpointKind::WPVram {  },
                // Category::HwReg => WatchpointKind::WPHwReg {  },
                _ => {},
            };
        }
        
        // --- Specific Variant Editing ---
        match wp_kind {
            WatchpointKind::CpuReg { .. } => {
                self.cpu_reg_wp_edit(ui, id, snem_core);
            }

            WatchpointKind::CpuFlag { .. } => {
                self.cpu_flag_wp_edit(ui, id, snem_core);
            }
            
            WatchpointKind::System { .. } => {
                self.system_wp_edit(ui, id, snem_core);
            }
        }
    }
    
    fn draw_log_editor(&mut self, ui: &mut egui::Ui, node_id: NodeId) {
        let node = match self.graph.nodes.get_mut(node_id) {
            Some(n) => n,
            None => return,
        };

        let NodeKind::Log(log_kind) = &mut node.kind else {
            return;
        };
        
        let mut current_cat = match log_kind {
            LogKind::CpuReg { .. } => RegCategory::CpuReg,
            LogKind::CpuFlag { .. } => RegCategory::Flag,
            LogKind::System { .. } => RegCategory::SysInfo,
        };
        let old_cat = current_cat;
        
        Self::draw_category_selector(ui, &mut current_cat);
        
        if old_cat != current_cat {
            let old_msg = match log_kind {
                LogKind::CpuReg { msg, .. } => msg.clone(),
                LogKind::CpuFlag { msg, .. } => msg.clone(),
                LogKind::System { msg, .. } => msg.clone(),
            };
            
            match current_cat {
                RegCategory::CpuReg => *log_kind = LogKind::CpuReg { reg: CpuReg::A, msg: old_msg },
                RegCategory::Flag => *log_kind = LogKind::CpuFlag { flag: CpuFlag::C, msg: old_msg },
                RegCategory::SysInfo => *log_kind = LogKind::System { variable: SystemVariable::Frame, msg: old_msg },
                // Category::Ram => WatchpointKind::WPRam {  },
                // Category::Vram => WatchpointKind::WPVram {  },
                // Category::HwReg => WatchpointKind::WPHwReg {  },
                _ => {},
            };
        }
        
        let mut message: Option<&mut String> = None;
        ui.horizontal(|ui| {
            ui.label(monospace_text("Log the value of".to_string()));
            
            match log_kind {
                LogKind::CpuReg { reg, msg } => {
                    Self::draw_cpu_reg_selector(ui, reg);
                    
                    message = Some(msg);
                }
                LogKind::CpuFlag { flag, msg } => {
                    Self::draw_cpu_flag_selector(ui, flag);
                    
                    message = Some(msg);
                }
                LogKind::System { variable, msg } => {
                    Self::draw_system_variable_selector(ui, variable);
                    
                    message = Some(msg);
                }
            }
        });
        
        ui.separator();
        
        ui.add(
            egui::TextEdit::singleline(message.unwrap()).hint_text("Message...")
        );
    }
    
    fn cpu_reg_wp_edit(&mut self, ui: &mut egui::Ui, wp_node_id: NodeId, snem_core: &snemcore::Snemulator) {
        enum RegSize { Byte, Word }
        
        let node = match self.graph.nodes.get_mut(wp_node_id) {
            Some(n) => n,
            None => return,
        };

        // 2. Ensure it's actually a Condition node
        let NodeKind::Condition(wp_kind) = &mut node.kind else {
            return;
        };
        
        let mut arg1: Option<usize>;
        let mut arg2: Option<usize>;
        let (mut reg, mut cond, reg_size) = match wp_kind {
            WatchpointKind::CpuReg { reg, cond } => {
                let size = match reg {
                    CpuReg::PB | CpuReg::DB | CpuReg::P => RegSize::Byte,
                    _ => RegSize::Word,
                };
                let c = match cond {
                    WatchpointCond::Equal(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::Eq
                    },
                    WatchpointCond::NotEqual(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::NEq
                    },
                    WatchpointCond::GreaterThan(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::Gt
                    },
                    WatchpointCond::LessThan(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::GtEq
                    },
                    WatchpointCond::OrEqual(cond_arg1, cond_arg2) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = Some(*cond_arg2);
                        RegCondType::OrEq
                    },
                    WatchpointCond::AndEqual(cond_arg1, cond_arg2) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = Some(*cond_arg2);
                        RegCondType::AndEq
                    },
                    WatchpointCond::Changed(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::Changed
                    }
                };
                
                (reg, c, size)
            }
            _ => unreachable!(),
        };
        let old_cond = cond.clone();
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));
            
            Self::draw_cpu_reg_selector(ui, &mut reg);
            
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("reg_cond").width(20.0)
                    .selected_text(
                        match cond {
                            RegCondType::Eq => "==",
                            RegCondType::NEq => "!=",
                            RegCondType::Gt => ">",
                            RegCondType::GtEq => ">=",
                            RegCondType::AndEq => "&",
                            RegCondType::OrEq => "|",
                            RegCondType::Changed => "Changed",
                        }
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cond, RegCondType::Eq, "== (Equals)");
                        ui.selectable_value(&mut cond, RegCondType::NEq, "!= (Not Equal)");
                        ui.selectable_value(&mut cond, RegCondType::Gt, "> (Greater Than)");
                        ui.selectable_value(&mut cond, RegCondType::GtEq, ">= (Greater Than or Equal)");
                        ui.selectable_value(&mut cond, RegCondType::AndEq, "& (Bitwise AND)");
                        ui.selectable_value(&mut cond, RegCondType::OrEq, "| (Bitwise OR)");
                        ui.selectable_value(&mut cond, RegCondType::Changed, "Changed");
                    });
            });
            
            let desired_width = match reg_size {
                RegSize::Byte => 20.0,
                RegSize::Word => 40.0,
            };
            let arg1_hint_text = match reg_size {
                RegSize::Byte => format!("{:02x}", arg1.unwrap_or_default()),
                RegSize::Word => format!("{:04x}", arg1.unwrap_or_default()),
            };
            let arg2_hint_text = match reg_size {
                RegSize::Byte => format!("{:02x}", arg2.unwrap_or_default()),
                RegSize::Word => format!("{:04x}", arg2.unwrap_or_default()),
            };
            
            ui.horizontal(|ui| {
                match cond {
                    RegCondType::Eq | RegCondType::NEq | RegCondType::Gt | RegCondType::GtEq => {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.cond_edit_arg1_text)
                                .desired_width(desired_width)
                                .hint_text(arg1_hint_text)
                        );
                        
                        let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if submitted {
                            match reg_size {
                                RegSize::Byte => {
                                    if let Ok(val) = u8::from_str_radix(self.cond_edit_arg1_text.trim(), 16) {
                                        arg1 = Some(val as usize);
                                    }
                                },
                                RegSize::Word => {
                                    if let Ok(val) = u16::from_str_radix(self.cond_edit_arg1_text.trim(), 16) {
                                        arg1 = Some(val as usize);
                                    }
                                },
                            }
                            self.cond_edit_arg1_text.clear()
                        }
            
                        self.editing_text = response.has_focus();
                    },
                    RegCondType::OrEq | RegCondType::AndEq => {
                        ui.horizontal(|ui| {                        
                            let arg1_response = ui.add(
                                egui::TextEdit::singleline(&mut self.cond_edit_arg1_text)
                                    .desired_width(desired_width)
                                    .hint_text(arg1_hint_text)
                            );
                            
                            ui.label(monospace_text("==".to_string()));
                            
                            let arg2_response = ui.add(
                                egui::TextEdit::singleline(&mut self.cond_edit_arg2_text)
                                    .desired_width(desired_width)
                                    .hint_text(arg2_hint_text)
                            );
                            
                            let arg1_submitted = arg1_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if arg1_submitted {
                                match reg_size {
                                    RegSize::Byte => {
                                        if let Ok(val) = u8::from_str_radix(self.cond_edit_arg1_text.trim(), 16) {
                                            arg1 = Some(val as usize);
                                        }
                                    },
                                    RegSize::Word => {
                                        if let Ok(val) = u16::from_str_radix(self.cond_edit_arg1_text.trim(), 16) {
                                            arg1 = Some(val as usize);
                                        }
                                    },
                                }
                                self.cond_edit_arg1_text.clear()
                            }
                            
                            let arg2_submitted = arg2_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if arg2_submitted {
                                match reg_size {
                                    RegSize::Byte => {
                                        if let Ok(val) = u8::from_str_radix(self.cond_edit_arg2_text.trim(), 16) {
                                            arg2 = Some(val as usize);
                                        }
                                    },
                                    RegSize::Word => {
                                        if let Ok(val) = u16::from_str_radix(self.cond_edit_arg2_text.trim(), 16) {
                                            arg2 = Some(val as usize);
                                        }
                                    },
                                }
                                self.cond_edit_arg2_text.clear()
                            }
                            
                            self.editing_text = arg1_response.has_focus() || arg2_response.has_focus();
                        });
                    },
                    RegCondType::Changed => {},
                }
            });
        });
        
        let reg = reg.clone();
        let cond = match cond {
            RegCondType::Eq => WatchpointCond::Equal(arg1.unwrap_or_default()),
            RegCondType::NEq => WatchpointCond::NotEqual(arg1.unwrap_or_default()),
            RegCondType::Gt => WatchpointCond::GreaterThan(arg1.unwrap_or_default()),
            RegCondType::GtEq => WatchpointCond::LessThan(arg1.unwrap_or_default()),
            RegCondType::OrEq => WatchpointCond::OrEqual(arg1.unwrap_or_default(), arg2.unwrap_or_default()),
            RegCondType::AndEq => WatchpointCond::AndEqual(arg1.unwrap_or_default(), arg2.unwrap_or_default()),
            RegCondType::Changed => {
                WatchpointCond::Changed(
                    if old_cond == cond {
                        arg1.unwrap()
                    } else {
                        match reg {
                            CpuReg::A => snem_core.cpu.a as usize,
                            CpuReg::X => snem_core.cpu.x as usize,
                            CpuReg::Y => snem_core.cpu.y as usize,
                            CpuReg::DB => snem_core.cpu.db as usize,
                            CpuReg::DP => snem_core.cpu.dp as usize,
                            CpuReg::PB => snem_core.cpu.pb as usize,
                            CpuReg::PC => snem_core.cpu.pc as usize,
                            CpuReg::SP => snem_core.cpu.sp as usize,
                            CpuReg::P => snem_core.cpu.p as usize,
                        }
                    }
                )
            },
        };
        
        let new_wp = WatchpointKind::CpuReg { reg, cond };
        
        *wp_kind = new_wp;
    }
    
    fn cpu_flag_wp_edit(&mut self, ui: &mut egui::Ui, wp_node_id: NodeId, snem_core: &snemcore::Snemulator) {
        let node = match self.graph.nodes.get_mut(wp_node_id) {
            Some(n) => n,
            None => return,
        };

        let NodeKind::Condition(wp_kind) = &mut node.kind else {
            return;
        };
        
        let (flag, cond) = match wp_kind {
            WatchpointKind::CpuFlag { flag, cond } => (flag, cond),
            _ => return,
        };

        let mut old_changed = None;
        match cond {
            WatchpointCondFlag::Changed(prev) => {
                old_changed = Some(*prev);
            }
            _ => {}
        }
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));
            
            Self::draw_cpu_flag_selector(ui, flag);
            
            ui.label(monospace_text("is".to_string()));
            
            egui::ComboBox::from_id_salt("flag_cond").width(20.0)
                .selected_text(match cond {
                    WatchpointCondFlag::Set => "Set",
                    WatchpointCondFlag::Clear => "Clear",
                    WatchpointCondFlag::Changed(_) => "Changed",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(cond, WatchpointCondFlag::Set,   "Set"  );
                    ui.selectable_value(cond, WatchpointCondFlag::Clear, "Clear");
                    ui.selectable_value(cond, WatchpointCondFlag::Changed(
                        old_changed.unwrap_or(match flag {
                            CpuFlag::C => snem_core.cpu.is_flag_set(scpu::Flag::FlagC),
                            CpuFlag::Z => snem_core.cpu.is_flag_set(scpu::Flag::FlagZ),
                            CpuFlag::I => snem_core.cpu.is_flag_set(scpu::Flag::FlagI),
                            CpuFlag::D => snem_core.cpu.is_flag_set(scpu::Flag::FlagD),
                            CpuFlag::X => snem_core.cpu.is_flag_set(scpu::Flag::FlagX),
                            CpuFlag::M => snem_core.cpu.is_flag_set(scpu::Flag::FlagM),
                            CpuFlag::V => snem_core.cpu.is_flag_set(scpu::Flag::FlagV),
                            CpuFlag::N => snem_core.cpu.is_flag_set(scpu::Flag::FlagN),
                            CpuFlag::Stopped => snem_core.cpu.stopped,
                            CpuFlag::Halted => snem_core.cpu.halted,
                            CpuFlag::Waiting => snem_core.cpu.waiting_for_interrupt,
                            CpuFlag::NMIPending => snem_core.cpu.nmi_pending,
                            CpuFlag::IRQPending => snem_core.cpu.irq_pending,
                        })
                    ), "Changed");
                });
        });
        
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
    
    fn system_wp_edit(&mut self, ui: &mut egui::Ui, wp_node_id: NodeId, snem_core: &snemcore::Snemulator) {
        let node = match self.graph.nodes.get_mut(wp_node_id) {
            Some(n) => n,
            None => return,
        };
    
        // 2. Ensure it's actually a Condition node
        let NodeKind::Condition(wp_kind) = &mut node.kind else {
            return;
        };
        
        let mut arg1: Option<usize>;
        let mut arg2: Option<usize>;
        let (mut variable, mut cond) = match wp_kind {
            WatchpointKind::System { variable, cond } => {
                let c = match cond {
                    WatchpointCond::Equal(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::Eq
                    },
                    WatchpointCond::NotEqual(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::NEq
                    },
                    WatchpointCond::GreaterThan(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::Gt
                    },
                    WatchpointCond::LessThan(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::GtEq
                    },
                    WatchpointCond::OrEqual(cond_arg1, cond_arg2) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = Some(*cond_arg2);
                        RegCondType::OrEq
                    },
                    WatchpointCond::AndEqual(cond_arg1, cond_arg2) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = Some(*cond_arg2);
                        RegCondType::AndEq
                    },
                    WatchpointCond::Changed(cond_arg1) => {
                        arg1 = Some(*cond_arg1);
                        arg2 = None;
                        RegCondType::Changed
                    },
                };
                
                (variable, c)
            }
            _ => unreachable!(),
        };
        let old_cond = cond.clone();
        
        ui.horizontal(|ui| {
            ui.label(monospace_text("If".to_string()));
            
            Self::draw_system_variable_selector(ui, &mut variable);
    
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("reg_cond").width(20.0)
                    .selected_text(
                        match cond {
                            RegCondType::Eq => "==",
                            RegCondType::NEq => "!=",
                            RegCondType::Gt => ">",
                            RegCondType::GtEq => ">=",
                            RegCondType::Changed => "changed",
                            _ => "",
                        }
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cond, RegCondType::Eq, "== (Equals)");
                        ui.selectable_value(&mut cond, RegCondType::NEq, "!= (Not Equal)");
                        ui.selectable_value(&mut cond, RegCondType::Gt, "> (Greater Than)");
                        ui.selectable_value(&mut cond, RegCondType::GtEq, ">= (Greater Than or Equal)");
                        ui.selectable_value(&mut cond, RegCondType::Changed, "Changed");
                    });
            });
            
            let desired_width = 40.0;
            let arg1_hint_text = format!("{}", arg1.unwrap_or_default());
            let arg2_hint_text = format!("{}", arg2.unwrap_or_default());
            
            ui.horizontal(|ui| {
                match cond {
                    RegCondType::Eq | RegCondType::NEq | RegCondType::Gt | RegCondType::GtEq => {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.cond_edit_arg1_text)
                                .desired_width(desired_width)
                                .hint_text(arg1_hint_text)
                        );
                        
                        let submitted = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if submitted {
                            if let Ok(val) = usize::from_str_radix(self.cond_edit_arg1_text.trim(), 10) {
                                arg1 = Some(val);
                            }
                            self.cond_edit_arg1_text.clear()
                        }
            
                        self.editing_text = response.has_focus();
                    }
                    RegCondType::OrEq | RegCondType::AndEq => {
                        ui.horizontal(|ui| {                        
                            let arg1_response = ui.add(
                                egui::TextEdit::singleline(&mut self.cond_edit_arg1_text)
                                    .desired_width(desired_width)
                                    .hint_text(arg1_hint_text)
                            );
                            
                            ui.label(monospace_text("==".to_string()));
                            
                            let arg2_response = ui.add(
                                egui::TextEdit::singleline(&mut self.cond_edit_arg2_text)
                                    .desired_width(desired_width)
                                    .hint_text(arg2_hint_text)
                            );
                            
                            let arg1_submitted = arg1_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if arg1_submitted {
                                if let Ok(val) = usize::from_str_radix(self.cond_edit_arg1_text.trim(), 10) {
                                    arg1 = Some(val);
                                }
                                self.cond_edit_arg1_text.clear()
                            }
                            
                            let arg2_submitted = arg2_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if arg2_submitted {
                                if let Ok(val) = usize::from_str_radix(self.cond_edit_arg2_text.trim(), 10) {
                                    arg2 = Some(val);
                                }
                                self.cond_edit_arg2_text.clear()
                            }
                            
                            self.editing_text = arg1_response.has_focus() || arg2_response.has_focus();
                        });
                    }
                    RegCondType::Changed => {}
                }
            });
        });
        
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
        
        
        let variable = variable.clone();
        let cond = match cond {
            RegCondType::Eq => WatchpointCond::Equal(arg1.unwrap_or_default()),
            RegCondType::NEq => WatchpointCond::NotEqual(arg1.unwrap_or_default()),
            RegCondType::Gt => WatchpointCond::GreaterThan(arg1.unwrap_or_default()),
            RegCondType::GtEq => WatchpointCond::LessThan(arg1.unwrap_or_default()),
            RegCondType::OrEq => WatchpointCond::OrEqual(arg1.unwrap_or_default(), arg2.unwrap_or_default()),
            RegCondType::AndEq => WatchpointCond::AndEqual(arg1.unwrap_or_default(), arg2.unwrap_or_default()),
            RegCondType::Changed => {
                WatchpointCond::Changed(
                    if old_cond == cond {
                        arg1.unwrap()
                    } else {
                        match variable {
                            SystemVariable::Frame => snem_core.frame as usize,
                            SystemVariable::Dot => snem_core.ppu.dot,
                            SystemVariable::Scanline => snem_core.ppu.scanline,
                        }
                    }
                )
            }
        };
        
        let new_wp = WatchpointKind::System { variable, cond };
        
        *wp_kind = new_wp;
    }

    // ── Interaction processing ────────────────────────────────────────────────

    /// Processes interactions with the canvas (dragging, selecting, etc.) and returns `true` if the graph was modified.
    fn process_interactions(
        &mut self,
        painter: &egui::Painter,
        response: &egui::Response,
        origin: Pos2,
        shift_held: bool,
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
            let Some(node) = self.graph.nodes.get(id) else { continue };

            let node_rect_screen = Rect::from_min_size(
                self.to_screen(origin, node.pos),
                egui::vec2(Node::WIDTH * self.zoom, Node::HEIGHT * self.zoom),
            );

            // ── Drag start (in Select mode only) ─────────────────────────────
            if matches!(self.drag, DragState::Idle) && response.drag_started_by(egui::PointerButton::Primary) {                
                let node = self.graph.nodes.get(id).unwrap();

                // Check output ports first — they take priority over node body.
                let mut started_wire = false;
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
            if matches!(self.drag, DragState::Idle) && response.clicked_by(egui::PointerButton::Primary) && !shift_held {
                let node = self.graph.nodes.get(id).unwrap();
                for i in 0..node.kind.input_count() {
                    let pp = self.to_screen(origin, node.input_port_pos(i));
                    if pp.distance(pointer_screen) < PORT_HIT_RADIUS {
                        let target = Port::new(id, i);
                        if let Some(wire_idx) =
                            self.graph.wires.iter().position(|w| w.to == target)
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
            if response.clicked_by(egui::PointerButton::Primary) && node_rect_screen.contains(pointer_screen) {
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
        if matches!(self.drag, DragState::Idle) && response.drag_started_by(egui::PointerButton::Primary) {
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

    fn draw_node(&self, painter: &egui::Painter, id: NodeId, node: &Node, rect: Rect, origin: Pos2) {
        let base_color = node.kind.color();
        let is_selected = self.selection.contains_node(id);

        let fill = if let NodeKind::Break { lit: true } = node.kind {
            Color32::from_rgb(255, 100, 80)
        } else {
            base_color.linear_multiply(0.25)
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
        let border_color = if is_selected { Color32::WHITE } else { base_color };
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(if is_selected { 2.5 } else { 1.5 }, border_color),
            egui::StrokeKind::Middle,
        );

        let font_size = 13.0 * self.zoom;
        let label = match &node.kind {
            NodeKind::Condition(wp_kind) => {
                &wp_kind.label()
            }
            NodeKind::Log(log_kind) => {
                &log_kind.label()
            }
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
            let signal = self.signals.get(&Port::new(id, o)).copied().unwrap_or(false);
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
