use std::{cell::Cell, collections::HashMap};

use snemcore::debug::watchpoints::{CompiledGraph, Counter, CounterMode, FastOp, Logpoint, NodeId, Watchpoint, WatchpointCond};

/// What kind of logic node this is.
#[derive(Clone)]
pub enum NodeKind {
    /// A togglable input switch. Has 0 inputs, 1 output.
    Condition(Watchpoint),
    /// AND gate. Has 2 inputs, 1 output.
    And,
    /// OR gate. Has 2 inputs, 1 output.
    Or,
    /// NOT gate. Has 1 input, 1 output.
    Not,
    /// Counter. Has 1 input, 1 output.
    Counter(Counter),
    /// Break indicator. Has 1 input, 0 outputs.
    Break { lit: bool },
    /// Log indicator. Has 1 input, 0 outputs.
    Log(Logpoint),
}

impl NodeKind {
    pub fn input_count(&self) -> usize {
        match self {
            NodeKind::Condition { .. } => 0,
            NodeKind::And | NodeKind::Or => 2,
            NodeKind::Not => 1,
            NodeKind::Counter { .. } => 1,
            NodeKind::Break { .. } => 1,
            NodeKind::Log { .. } => 1,
        }
    }

    pub fn output_count(&self) -> usize {
        match self {
            NodeKind::Condition { .. } => 1,
            NodeKind::And | NodeKind::Or | NodeKind::Not | NodeKind::Counter { .. } => 1,
            NodeKind::Break { .. } => 0,
            NodeKind::Log { .. } => 0,
        }
    }

    pub fn label(&self) -> String {
        match self {
            NodeKind::Condition { .. } => "".to_string(),
            NodeKind::And => "AND".to_string(),
            NodeKind::Or => "OR".to_string(),
            NodeKind::Not => "NOT".to_string(),
            NodeKind::Counter(cnt) => cnt.label(),
            NodeKind::Break { .. } => "Break".to_string(),
            NodeKind::Log { .. } => "".to_string(),
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            NodeKind::Condition { .. } => egui::Color32::from_rgb(0xBD, 0xB2, 0xFF),
            NodeKind::And => egui::Color32::from_rgb(0xCA, 0xFF, 0xBF),
            NodeKind::Or => egui::Color32::from_rgb(0x9B, 0xF6, 0xFF),
            NodeKind::Not => egui::Color32::from_rgb(0xFF, 0xC6, 0xFF),
            NodeKind::Counter { .. } => egui::Color32::from_rgb(0xFF, 0xD6, 0xA5),
            NodeKind::Log { .. } => egui::Color32::from_rgb(0xDC, 0xDC, 0xDC),
            NodeKind::Break { .. } => egui::Color32::from_rgb(0xFF, 0xAD, 0xAD),
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
    pub nodes: slotmap::SlotMap<NodeId, Node>,
    pub wires: Vec<Wire>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: slotmap::SlotMap::with_key(),
            wires: Vec::new(),
        }
    }

    pub fn add_node(&mut self, kind: NodeKind, pos: egui::Pos2) -> NodeId {
        self.nodes.insert(Node::new(kind, pos))
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(id);
        self.wires.retain(|w| w.from.node != id && w.to.node != id);
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
    pub fn evaluate(&mut self, snem_core: &snemcore::Snemulator) -> HashMap<Port, bool> {
        let order = self.topological_order();
        let mut signals: HashMap<Port, bool> = HashMap::new();

        // Seed inputs from InputSwitch nodes.
        for id in &order {
            if let Some(node) = self.nodes.get_mut(*id) {
                if let NodeKind::Condition(cond) = &mut node.kind {
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

            let output = match &mut node.kind {
                NodeKind::Condition(wp) => Some(wp.evaluate(snem_core)),
                NodeKind::And => Some(inputs.iter().all(|&b| b)),
                NodeKind::Or => Some(inputs.iter().any(|&b| b)),
                NodeKind::Not => Some(!inputs.first().copied().unwrap_or(false)),
                NodeKind::Counter(cnt) => Some(cnt.evaluate()),
                NodeKind::Break { .. } => {
                    let val = inputs.first().copied().unwrap_or(false);
                    node.kind = NodeKind::Break { lit: val };
                    None
                }
                NodeKind::Log { .. } => None,
            };

            if let Some(val) = output {
                signals.insert(Port::new(*id, 0), val);
            }
        }

        signals
    }

    /// Kahn's algorithm for topological sort. Cycles are broken by skipping them.
    fn topological_order(&self) -> Vec<NodeId> {
        let mut in_degree: HashMap<NodeId, usize> = self.nodes.keys().map(|id| (id, 0)).collect();

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

    pub fn compile(&mut self, snem_core: &snemcore::Snemulator) -> CompiledGraph {
        // Index 0 is always a fallback 'false' for unconnected input ports.
        let mut ops = vec![FastOp::Constant(false)];
        let mut node_to_idx = HashMap::new();

        for id in self.topological_order() {
            let node = match self.nodes.get_mut(id) {
                Some(n) => n,
                None => continue,
            };

            // Map inputs to the index of previously evaluated results
            let inputs: Vec<usize> = (0..node.kind.input_count())
                .map(|i| {
                    let target = Port::new(id, i);
                    self.wires
                        .iter()
                        .find(|w| w.to == target)
                        .and_then(|w| node_to_idx.get(&w.from.node).copied())
                        .unwrap_or(0) // Default to index 0 (FastOp::Constant(false))
                })
                .collect();

            let op = match &mut node.kind {
                NodeKind::Condition(wp) => match &mut wp.cond {
                    WatchpointCond::Changed => {
                        wp.arg1 = wp.val.get_value(snem_core);
                        FastOp::CondChanged {
                            prev_value: Cell::new(wp.arg1),
                            value: wp.val.clone(),
                            id: id,
                        }
                    }
                    _ => FastOp::Cond(wp.clone()),
                },
                NodeKind::And => FastOp::And(inputs[0], inputs[1]),
                NodeKind::Or => FastOp::Or(inputs[0], inputs[1]),
                NodeKind::Not => FastOp::Not(inputs[0]),
                NodeKind::Counter(cnt) => match cnt.mode {
                    CounterMode::IncOnChange => FastOp::CounterRisingEdge {
                        input: inputs[0],
                        prev: Cell::new(cnt.prev),
                        count: Cell::new(cnt.count),
                        arg: cnt.arg,
                        cond: cnt.cond.clone(),
                        reset: cnt.reset,
                        fired: Cell::new(false),
                        id,
                    },
                    CounterMode::IncOnTrue => FastOp::CounterHigh {
                        input: inputs[0],
                        count: Cell::new(cnt.count),
                        arg: cnt.arg,
                        cond: cnt.cond.clone(),
                        reset: cnt.reset,
                        fired: Cell::new(false),
                        id,
                    },
                },
                NodeKind::Break { .. } => FastOp::Output(inputs[0]),
                NodeKind::Log(lp) => FastOp::Log(inputs[0], lp.clone()),
                // _ => FastOp::Constant(false),
            };

            node_to_idx.insert(id, ops.len());
            ops.push(op);
        }

        CompiledGraph::new(ops)
    }
}