use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

use crate::core::{self, scpu};

new_key_type! { pub struct NodeId; }

#[derive(Clone, Debug, PartialEq)]
pub enum CpuRegU8 {
    DB, PB, P,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CpuRegU16 {
    A, X, Y, DP, PC, SP,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WatchpointCondU8 {
    Equal(u8),
    AndEqual(u8, u8),
    OrEqual(u8, u8),
    // Changes,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WatchpointCondU16 {
    Equal(u16),
    AndEqual(u16, u16),
    OrEqual(u16, u16),
    // Changes
}

#[derive(Clone, PartialEq)]
pub enum WatchpointCondFlag {
    Equal(bool),
    // Changes
}

#[derive(Clone)]
pub enum WatchpointKind {
    WPCpuReg8 {
        reg: CpuRegU8,
        cond: WatchpointCondU8,
    },
    WPCpuReg16 {
        reg: CpuRegU16,
        cond: WatchpointCondU16,
    },
    WPCpuFlag {
        flag: scpu::Flag,
        cond: WatchpointCondFlag,
    }
}

impl WatchpointKind {
    fn evaluate(&self, snem_core: &core::snemcore::Snemulator) -> bool {
        match self {
            WatchpointKind::WPCpuReg8 { reg, cond } => {
                let reg = match reg {
                    CpuRegU8::DB => snem_core.cpu.db,
                    CpuRegU8::P => snem_core.cpu.p,
                    CpuRegU8::PB => snem_core.cpu.pb,
                };
                match cond {
                    WatchpointCondU8::Equal(val) => reg == *val,
                    WatchpointCondU8::AndEqual(val1, val2) => (reg & *val1) == *val2,
                    WatchpointCondU8::OrEqual(val1, val2) => (reg | *val1) == *val2,
                }
            },
            WatchpointKind::WPCpuReg16 { reg, cond } => {
                let reg = match reg {
                    CpuRegU16::A => snem_core.cpu.a,
                    CpuRegU16::X => snem_core.cpu.x,
                    CpuRegU16::Y => snem_core.cpu.y,
                    CpuRegU16::DP => snem_core.cpu.dp,
                    CpuRegU16::PC => snem_core.cpu.pc,
                    CpuRegU16::SP => snem_core.cpu.sp,
                };
                match cond {
                    WatchpointCondU16::Equal(val) => reg == *val,
                    WatchpointCondU16::AndEqual(val1, val2) => (reg & *val1) == *val2,
                    WatchpointCondU16::OrEqual(val1, val2) => (reg | *val1) == *val2,
                }
            },
            WatchpointKind::WPCpuFlag { flag, cond } => {
                let flag = snem_core.cpu.is_flag_set(*flag);
                match cond {
                    WatchpointCondFlag::Equal(val) => flag == *val,
                }
            },
        }
    }
    
    pub fn label(&self) -> String {
        match self {
            WatchpointKind::WPCpuReg8 { reg, cond } => {
                format!("{} {}",
                    match reg {
                        CpuRegU8::DB => "DB",
                        CpuRegU8::P => "P",
                        CpuRegU8::PB => "PB",
                    },
                    match cond {
                        WatchpointCondU8::Equal(val) => format!("== {}", val),
                        WatchpointCondU8::AndEqual(val1, val2) => format!("& {} == {}", val1, val2),
                        WatchpointCondU8::OrEqual(val1, val2) => format!("| {} == {}", val1, val2),
                    },
                )
            },
            WatchpointKind::WPCpuReg16 { reg, cond } => {
                format!("{} {}",
                    match reg {
                        CpuRegU16::A => "A",
                        CpuRegU16::X => "X",
                        CpuRegU16::Y => "Y",
                        CpuRegU16::DP => "DP",
                        CpuRegU16::PC => "PC",
                        CpuRegU16::SP => "SP",
                    },
                    match cond {
                        WatchpointCondU16::Equal(val) => format!("== {}", val),
                        WatchpointCondU16::AndEqual(val1, val2) => format!("& {} == {}", val1, val2),
                        WatchpointCondU16::OrEqual(val1, val2) => format!("| {} == {}", val1, val2),
                    }
                )
            },
            WatchpointKind::WPCpuFlag { flag, cond } => {
                format!(
                    "CPU Flag {} is {}",
                    match flag {
                        scpu::Flag::FlagC => "C",
                        scpu::Flag::FlagZ => "Z",
                        scpu::Flag::FlagI => "I",
                        scpu::Flag::FlagD => "D",
                        scpu::Flag::FlagX => "X",
                        scpu::Flag::FlagM => "M",
                        scpu::Flag::FlagV => "V",
                        scpu::Flag::FlagN => "N",
                    },
                    match cond {
                        WatchpointCondFlag::Equal(val) => if *val { "set" } else { "cleared" },
                    },
                )
            },
        }
    }
}

/// What kind of logic node this is.
#[derive(Clone)]
pub enum NodeKind {
    /// A togglable input switch. Has 0 inputs, 1 output.
    Condition(WatchpointKind),
    /// AND gate. Has 2 inputs, 1 output.
    And,
    /// OR gate. Has 2 inputs, 1 output.
    Or,
    /// NOT gate. Has 1 input, 1 output.
    Not,
    /// Output indicator. Has 1 input, 0 outputs.
    Output { lit: bool },
}

impl NodeKind {
    pub fn input_count(&self) -> usize {
        match self {
            NodeKind::Condition { .. } => 0,
            NodeKind::And | NodeKind::Or => 2,
            NodeKind::Not => 1,
            NodeKind::Output { .. } => 1,
        }
    }

    pub fn output_count(&self) -> usize {
        match self {
            NodeKind::Condition { .. } => 1,
            NodeKind::And | NodeKind::Or | NodeKind::Not => 1,
            NodeKind::Output { .. } => 0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            NodeKind::Condition { .. } => "Switch",
            NodeKind::And => "AND",
            NodeKind::Or => "OR",
            NodeKind::Not => "NOT",
            NodeKind::Output { .. } => "Output",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            NodeKind::Condition { .. } => egui::Color32::from_rgb(60, 120, 200),
            NodeKind::And => egui::Color32::from_rgb(80, 160, 80),
            NodeKind::Or => egui::Color32::from_rgb(160, 120, 40),
            NodeKind::Not => egui::Color32::from_rgb(160, 60, 160),
            NodeKind::Output { .. } => egui::Color32::from_rgb(200, 60, 60),
        }
    }
}

/// A node in the graph, with a position on the canvas.
#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    /// Top-left corner of the node in canvas space.
    pub pos: egui::Pos2,
}

impl Node {
    pub fn new(kind: NodeKind, pos: egui::Pos2) -> Self {
        Self { kind, pos }
    }

    pub const WIDTH: f32 = 110.0;
    pub const HEIGHT: f32 = 70.0;

    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(self.pos, egui::vec2(Self::WIDTH, Self::HEIGHT))
    }

    /// Position of an input port in canvas space.
    pub fn input_port_pos(&self, index: usize) -> egui::Pos2 {
        let count = self.kind.input_count();
        let step = Self::HEIGHT / (count + 1) as f32;
        self.pos + egui::vec2(0.0, step * (index + 1) as f32)
    }

    /// Position of an output port in canvas space.
    pub fn output_port_pos(&self, index: usize) -> egui::Pos2 {
        let count = self.kind.output_count();
        let step = Self::HEIGHT / (count + 1) as f32;
        self.pos + egui::vec2(Self::WIDTH, step * (index + 1) as f32)
    }
}

/// Identifies one end of a wire: a specific port on a specific node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Port {
    pub node: NodeId,
    pub port: usize,
}

impl Port {
    pub fn new(node: NodeId, port: usize) -> Self {
        Self { node, port }
    }
}

/// A directed connection from an output port to an input port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wire {
    pub from: Port, // output port
    pub to: Port,   // input port
}

/// The complete simulation graph.
pub struct Graph {
    pub nodes: SlotMap<NodeId, Node>,
    pub wires: Vec<Wire>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            wires: Vec::new(),
        }
    }

    pub fn add_node(&mut self, kind: NodeKind, pos: egui::Pos2) -> NodeId {
        self.nodes.insert(Node::new(kind, pos))
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(id);
        self.wires
            .retain(|w| w.from.node != id && w.to.node != id);
    }

    /// Add a wire, preventing duplicates and fan-in conflicts (one driver per input).
    pub fn add_wire(&mut self, wire: Wire) {
        // An input port can only have one driver.
        self.wires.retain(|w| w.to != wire.to);
        if !self.wires.contains(&wire) {
            self.wires.push(wire);
        }
    }

    /// Evaluate all node outputs using topological order.
    /// Returns a map from output Port -> bool signal value.
    pub fn evaluate(&mut self, snem_core: &core::snemcore::Snemulator) -> HashMap<Port, bool> {
        let order = self.topological_order();
        let mut signals: HashMap<Port, bool> = HashMap::new();

        // Seed inputs from InputSwitch nodes.
        for id in &order {
            if let Some(node) = self.nodes.get(*id) {
                if let NodeKind::Condition(cond) = &node.kind {
                    signals.insert(Port::new(*id, 0), cond.evaluate(snem_core));
                }
            }
        }

        // Evaluate in order.
        for id in &order {
            let inputs: Vec<bool> = {
                let node = match self.nodes.get(*id) {
                    Some(n) => n,
                    None => continue,
                };
                (0..node.kind.input_count())
                    .map(|i| {
                        let target = Port::new(*id, i);
                        self.wires
                            .iter()
                            .find(|w| w.to == target)
                            .and_then(|w| signals.get(&w.from))
                            .copied()
                            .unwrap_or(false)
                    })
                    .collect()
            };

            let node = match self.nodes.get_mut(*id) {
                Some(n) => n,
                None => continue,
            };

            let output = match &node.kind {
                NodeKind::Condition(cond) => Some(cond.evaluate(snem_core)),
                NodeKind::And => Some(inputs.iter().all(|&b| b)),
                NodeKind::Or => Some(inputs.iter().any(|&b| b)),
                NodeKind::Not => Some(!inputs.first().copied().unwrap_or(false)),
                NodeKind::Output { .. } => {
                    let val = inputs.first().copied().unwrap_or(false);
                    node.kind = NodeKind::Output { lit: val };
                    None
                }
            };

            if let Some(val) = output {
                signals.insert(Port::new(*id, 0), val);
            }
        }

        signals
    }

    /// Kahn's algorithm for topological sort. Cycles are broken by skipping them.
    fn topological_order(&self) -> Vec<NodeId> {
        let mut in_degree: HashMap<NodeId, usize> =
            self.nodes.keys().map(|id| (id, 0)).collect();

        for wire in &self.wires {
            *in_degree.entry(wire.to.node).or_insert(0) += 1;
        }

        let mut queue: std::collections::VecDeque<NodeId> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();

        while let Some(id) = queue.pop_front() {
            order.push(id);
            for wire in self.wires.iter().filter(|w| w.from.node == id) {
                let deg = in_degree.entry(wire.to.node).or_insert(0);
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push_back(wire.to.node);
                }
            }
        }

        // Append any remaining nodes (cycle participants) so they still get evaluated.
        for id in self.nodes.keys() {
            if !order.contains(&id) {
                order.push(id);
            }
        }

        order
    }
    
    pub fn compile(&self) -> CompiledGraph {
        // Index 0 is always a fallback 'false' for unconnected input ports.
        let mut ops = vec![FastOp::Constant(false)]; 
        let mut node_to_idx = HashMap::new();

        for id in self.topological_order() {
            let node = match self.nodes.get(id) {
                Some(n) => n,
                None => continue,
            };

            // Map inputs to the index of previously evaluated results
            let inputs: Vec<usize> = (0..node.kind.input_count()).map(|i| {
                let target = Port::new(id, i);
                self.wires.iter()
                    .find(|w| w.to == target)
                    .and_then(|w| node_to_idx.get(&w.from.node).copied())
                    .unwrap_or(0) // Default to index 0 (FastOp::Constant(false))
            }).collect();

            let op = match &node.kind {
                NodeKind::Condition(cond) => FastOp::Cond(cond.clone()),
                NodeKind::And => FastOp::And(inputs[0], inputs[1]),
                NodeKind::Or  => FastOp::Or(inputs[0], inputs[1]),
                NodeKind::Not => FastOp::Not(inputs[0]),
                NodeKind::Output { .. } => FastOp::Output(inputs[0]),
                _ => FastOp::Constant(false),
            };

            node_to_idx.insert(id, ops.len());
            ops.push(op);
        }

        CompiledGraph { ops }
    }
}


// Add this to types.rs
#[derive(Clone)]
pub enum FastOp {
    Constant(bool),
    Cond(WatchpointKind),
    And(usize, usize),
    Or(usize, usize),
    Not(usize),
    Output(usize),
}

#[derive(Clone, Default)]
pub struct CompiledGraph {
    ops: Vec<FastOp>,
}

impl CompiledGraph {
    /// Extremely fast, allocation-free evaluation for the hot CPU loop.
    pub fn evaluate(&self, snem_core: &core::snemcore::Snemulator) -> bool {
        if self.ops.is_empty() { return false; }
        
        // Use a fixed-size array or small vec to store intermediate results
        let mut results = vec![false; self.ops.len()];
        let mut break_triggered = false;

        for (i, op) in self.ops.iter().enumerate() {
            results[i] = match op {
                FastOp::Constant(val) => *val,
                FastOp::Cond(cond) => cond.evaluate(snem_core),
                FastOp::And(a, b) => results[*a] && results[*b],
                FastOp::Or(a, b)  => results[*a] || results[*b],
                FastOp::Not(a)    => !results[*a],
                FastOp::Output(a) => {
                    if results[*a] { break_triggered = true; }
                    false
                }
            };
        }
        break_triggered
    }
}