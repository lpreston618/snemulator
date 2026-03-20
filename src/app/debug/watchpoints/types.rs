use std::{cell::Cell, collections::HashMap, str::FromStr};

use crate::core::{self, scpu, snemcore};

pub const HWVAL_NAMES: [&str; 10] = [
    // Regs
    "APUIO0", "APUIO1", "APUIO2", "APUIO3",
    "CPUIO0", "CPUIO1", "CPUIO2", "CPUIO3",
    // Flags
    "VBLANK", "FBLANK",
];

slotmap::new_key_type! { pub struct NodeId; }

#[derive(PartialEq, Clone, Copy)]
pub enum RegCategory { CpuReg, Flag, Ram, Vram, HwRegOrFlag, SysInfo, LogMessageOnly }

impl RegCategory {
    pub fn label(&self) -> &'static str {
        match self {
            RegCategory::CpuReg => "CPU Register",
            RegCategory::Flag => "CPU Flag",
            RegCategory::Ram => "RAM",
            RegCategory::Vram => "VRAM",
            RegCategory::HwRegOrFlag => "Hardware Register",
            RegCategory::SysInfo => "System Info",
            RegCategory::LogMessageOnly => "Log Message Only",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RegCondType { Eq, NEq, Gt, Lt, AndEq, OrEq, Changed }

impl RegCondType {
    pub fn selected_label(&self) -> &'static str {
        match self {
            RegCondType::Eq => "==",
            RegCondType::NEq => "!=",
            RegCondType::Gt => ">",
            RegCondType::Lt => "<",
            RegCondType::AndEq => "&",
            RegCondType::OrEq => "|",
            RegCondType::Changed => "Changed",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            RegCondType::Eq => "== (Equals)",
            RegCondType::NEq => "!= (Not Equal)",
            RegCondType::Gt => "> (Greater Than)",
            RegCondType::Lt => "< (Less Than)",
            RegCondType::AndEq => "& (Bitwise AND)",
            RegCondType::OrEq => "| (Bitwise OR)",
            RegCondType::Changed => "Changed",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RegSize { Byte, Word, Num }

#[derive(Clone, Debug, PartialEq)]
pub enum CpuReg {
    DB, PB, P, A, X, Y, DP, PC, SP,
}

impl CpuReg {
    pub fn get_value(&self, snem_core: &core::snemcore::Snemulator) -> usize {
        match self {
            CpuReg::DB => snem_core.cpu.db as usize,
            CpuReg::PB => snem_core.cpu.pb as usize,
            CpuReg::P => snem_core.cpu.p as usize,
            CpuReg::A => snem_core.cpu.a as usize,
            CpuReg::X => snem_core.cpu.x as usize,
            CpuReg::Y => snem_core.cpu.y as usize,
            CpuReg::DP => snem_core.cpu.dp as usize,
            CpuReg::PC => snem_core.cpu.pc as usize,
            CpuReg::SP => snem_core.cpu.sp as usize,
        }
    }
    
    pub fn label(&self) -> String {
        format!("{:?}", self)
    }
    
    fn reg_size(&self) -> RegSize {
        match self {
            CpuReg::DB | CpuReg::PB | CpuReg::P => RegSize::Byte,
            _ => RegSize::Word,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CpuFlag {
    C, Z, I, D, X, M, V, N,
    Stopped, Halted, Waiting,
    NMIPending, IRQPending,
}

impl CpuFlag {
    pub fn get_value(&self, snem_core: &core::snemcore::Snemulator) -> bool {
        match self {
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
        }
    }
    
    pub fn label(&self) -> String {
        match self {
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
        }.to_string()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HardwareReg {
    ApuIo0,
    ApuIo1,
    ApuIo2,
    ApuIo3,
    CpuIo0,
    CpuIo1,
    CpuIo2,
    CpuIo3,
}

impl FromStr for HardwareReg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "APUIO0" => Ok(HardwareReg::ApuIo0),
            "APUIO1" => Ok(HardwareReg::ApuIo1),
            "APUIO2" => Ok(HardwareReg::ApuIo2),
            "APUIO3" => Ok(HardwareReg::ApuIo3),
            "CPUIO0" => Ok(HardwareReg::CpuIo0),
            "CPUIO1" => Ok(HardwareReg::CpuIo1),
            "CPUIO2" => Ok(HardwareReg::CpuIo2),
            "CPUIO3" => Ok(HardwareReg::CpuIo3),
            _ => Err(()),
        }
    }
}

impl HardwareReg {
    pub fn get_value(&self, snem_core: &core::snemcore::Snemulator) -> usize {
        match self {
            HardwareReg::ApuIo0 => snem_core.apu_ports.apuio0 as usize,
            HardwareReg::ApuIo1 => snem_core.apu_ports.apuio1 as usize,
            HardwareReg::ApuIo2 => snem_core.apu_ports.apuio2 as usize,
            HardwareReg::ApuIo3 => snem_core.apu_ports.apuio3 as usize,
            HardwareReg::CpuIo0 => snem_core.apu_ports.cpuio0 as usize,
            HardwareReg::CpuIo1 => snem_core.apu_ports.cpuio1 as usize,
            HardwareReg::CpuIo2 => snem_core.apu_ports.cpuio2 as usize,
            HardwareReg::CpuIo3 => snem_core.apu_ports.cpuio3 as usize,
        }
    }
    
    pub fn label(&self) -> String {
        match self {
            HardwareReg::ApuIo0 => "APUIO0",
            HardwareReg::ApuIo1 => "APUIO1",
            HardwareReg::ApuIo2 => "APUIO2",
            HardwareReg::ApuIo3 => "APUIO3",
            HardwareReg::CpuIo0 => "CPUIO0",
            HardwareReg::CpuIo1 => "CPUIO1",
            HardwareReg::CpuIo2 => "CPUIO2",
            HardwareReg::CpuIo3 => "CPUIO3",
        }.to_string()
    }
    
    pub fn reg_size(&self) -> RegSize {
        match self {
            HardwareReg::ApuIo0 => RegSize::Byte,
            HardwareReg::ApuIo1 => RegSize::Byte,
            HardwareReg::ApuIo2 => RegSize::Byte,
            HardwareReg::ApuIo3 => RegSize::Byte,
            HardwareReg::CpuIo0 => RegSize::Byte,
            HardwareReg::CpuIo1 => RegSize::Byte,
            HardwareReg::CpuIo2 => RegSize::Byte,
            HardwareReg::CpuIo3 => RegSize::Byte,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HardwareFlag {
    VBlank,
    FBlank,
}

impl FromStr for HardwareFlag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VBLANK" => Ok(HardwareFlag::VBlank),
            "FBLANK" => Ok(HardwareFlag::FBlank),
            _ => Err(()),
        }
    }
}

impl HardwareFlag {
    pub fn get_value(&self, snem_core: &core::snemcore::Snemulator) -> bool {
        match self {
            HardwareFlag::VBlank => snem_core.cpu_regs.vblank_flag,
            HardwareFlag::FBlank => snem_core.ppu_regs.in_fblank,
        }
    }
    
    pub fn label(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SystemVariable {
    Frame,
    Dot,
    Scanline,
}

impl SystemVariable {
    pub fn get_value(&self, snem_core: &core::snemcore::Snemulator) -> usize {
        match self {
            SystemVariable::Frame => snem_core.frame as usize,
            SystemVariable::Dot => snem_core.ppu.dot as usize,
            SystemVariable::Scanline => snem_core.ppu.scanline as usize,
        }
    }
    
    pub fn label(&self) -> String {
        format!("{:?}", self)
    }
    
    fn variable_size(&self) -> RegSize {
        RegSize::Num
    }
}

#[derive(Clone, PartialEq)]
pub enum WatchpointCondFlag {
    Set,
    Clear,
    Changed(bool),
}

impl WatchpointCondFlag {
    pub fn evaluate(&self, value: bool) -> bool {
        match self {
            WatchpointCondFlag::Set => value,
            WatchpointCondFlag::Clear => !value,
            WatchpointCondFlag::Changed(prev) => value != *prev,
        }
    }
    
    pub fn label(&self) -> String {
        match self {
            WatchpointCondFlag::Set => "Set".to_string(),
            WatchpointCondFlag::Clear => "Clear".to_string(),
            WatchpointCondFlag::Changed(_) => "Changed".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WatchpointCond {
    Equal(usize),
    NotEqual(usize),
    GreaterThan(usize),
    LessThan(usize),
    AndEqual(usize, usize),
    OrEqual(usize, usize),
    Changed(usize),
}

impl WatchpointCond {
    pub fn evaluate(&self, value: usize) -> bool {
        match self {
            WatchpointCond::Equal(cond_val) => value == *cond_val,
            WatchpointCond::NotEqual(cond_val) => value != *cond_val,
            WatchpointCond::GreaterThan(cond_val) => value > *cond_val,
            WatchpointCond::LessThan(cond_val) => value < *cond_val,
            WatchpointCond::AndEqual(operand, cond_val) => value & *operand == *cond_val,
            WatchpointCond::OrEqual(operand, cond_val) => value | *operand == *cond_val,
            WatchpointCond::Changed(prev) => value != *prev,
        }
    }
    
    fn label(&self, num_cond_fmt: RegSize) -> String {
        match num_cond_fmt {
            RegSize::Byte => match self {
                WatchpointCond::Equal(val) => format!("== {:02X}", val),
                WatchpointCond::NotEqual(val) => format!("!= {:02X}", val),
                WatchpointCond::GreaterThan(val) => format!("> {:02X}", val),
                WatchpointCond::LessThan(val) => format!("< {:02X}", val),
                WatchpointCond::AndEqual(val1, val2) => format!("& {:02X}\n == {:02X}", val1, val2),
                WatchpointCond::OrEqual(val1, val2) => format!("| {:02X}\n == {:02X}", val1, val2),
                WatchpointCond::Changed(_) => "Changed".to_string(),
            },
            RegSize::Word => match self {
                WatchpointCond::Equal(val) => format!("== {:04X}", val),
                WatchpointCond::NotEqual(val) => format!("!= {:04X}", val),
                WatchpointCond::GreaterThan(val) => format!("> {:04X}", val),
                WatchpointCond::LessThan(val) => format!("< {:04X}", val),
                WatchpointCond::AndEqual(val1, val2) => format!("& {:04X}\n == {:04X}", val1, val2),
                WatchpointCond::OrEqual(val1, val2) => format!("| {:04X}\n == {:04X}", val1, val2),
                WatchpointCond::Changed(_) => "Changed".to_string(),
            },
            RegSize::Num => match self {
                WatchpointCond::Equal(val) => format!("== {}", val),
                WatchpointCond::NotEqual(val) => format!("!= {}", val),
                WatchpointCond::GreaterThan(val) => format!("> {}", val),
                WatchpointCond::LessThan(val) => format!("< {}", val),
                WatchpointCond::AndEqual(val1, val2) => format!("& {}\n == {}", val1, val2),
                WatchpointCond::OrEqual(val1, val2) => format!("| {}\n == {}", val1, val2),
                WatchpointCond::Changed(_) => "Changed".to_string(),
            },
        }
    }
}

#[derive(Clone)]
pub enum WatchpointKind {
    CpuReg {
        reg: CpuReg,
        cond: WatchpointCond,
    },
    CpuFlag {
        flag: CpuFlag,
        cond: WatchpointCondFlag,
    },
    HardwareReg {
        reg: HardwareReg,
        cond: WatchpointCond,
    },
    HardwareFlag {
        flag: HardwareFlag,
        cond: WatchpointCondFlag,
    },
    System {
        variable: SystemVariable,
        cond: WatchpointCond,
    }
}

impl Default for WatchpointKind {
    fn default() -> Self {
        Self::CpuReg {
            reg: CpuReg::A,
            cond: WatchpointCond::Equal(0),
        }
    }
}

impl WatchpointKind {
    pub fn evaluate(&self, snem_core: &core::snemcore::Snemulator) -> bool {
        match self {
            WatchpointKind::CpuReg { reg, cond } => {
                cond.evaluate(reg.get_value(snem_core))
            },
            WatchpointKind::CpuFlag { flag, cond } => {
                cond.evaluate(flag.get_value(snem_core))
            },
            WatchpointKind::HardwareReg { reg, cond } => {
                cond.evaluate(reg.get_value(snem_core))
            }
            WatchpointKind::HardwareFlag { flag, cond } => {
                cond.evaluate(flag.get_value(snem_core))
            }
            WatchpointKind::System { variable, cond } => {
                cond.evaluate(variable.get_value(snem_core))
            }
        }
    }
    
    pub fn label(&self) -> String {
        match self {
            WatchpointKind::CpuReg { reg, cond } => {
                let num_cond_fmt = reg.reg_size();
                
                format!("{} {}", reg.label(), cond.label(num_cond_fmt))
            },
            WatchpointKind::CpuFlag { flag, cond } => {
                match flag {
                    CpuFlag::C | CpuFlag::Z |
                    CpuFlag::I | CpuFlag::D |
                    CpuFlag::X | CpuFlag::M |
                    CpuFlag::V | CpuFlag::N => {
                        format!("CPU Status\nflag {:?} is\n{}", flag, cond.label())
                    },
                    _ => {
                        format!("CPU Flag\n{:?} is\n{}", flag, cond.label())
                    }
                }
            },
            WatchpointKind::HardwareReg { reg, cond } => {
                let num_cond_fmt = reg.reg_size();
                
                format!("{} {}", reg.label(), cond.label(num_cond_fmt))
            }
            WatchpointKind::HardwareFlag { flag, cond } => {
                format!("{} is {}", flag.label(), cond.label())
            },
            WatchpointKind::System { variable, cond } => {
                let num_cond_fmt = variable.variable_size();
                
                format!("{} {}", variable.label(), cond.label(num_cond_fmt))
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogKind {
    CpuReg {
        reg: CpuReg,
        msg: String,
    },
    CpuFlag {
        flag: CpuFlag,
        msg: String,
    },
    System {
        variable: SystemVariable,
        msg: String,
    },
    Message {
        msg: String,
    },
}

impl Default for LogKind {
    fn default() -> Self {
        Self::CpuReg {
            reg: CpuReg::A,
            msg: String::new(),
        }
    }
}

impl LogKind {
    pub fn message(&self, snem_core: &snemcore::Snemulator) -> String {
        match self {
            LogKind::CpuReg { reg, msg } => {
                enum RegSize { Byte, Word }
                
                let (reg_val, reg_size) = match reg {
                    CpuReg::DB => (snem_core.cpu.db as u16, RegSize::Byte),
                    CpuReg::PB => (snem_core.cpu.pb as u16, RegSize::Byte),
                    CpuReg::P => (snem_core.cpu.p as u16, RegSize::Byte),
                    CpuReg::A => (snem_core.cpu.a as u16, RegSize::Word),
                    CpuReg::X => (snem_core.cpu.x as u16, RegSize::Word),
                    CpuReg::Y => (snem_core.cpu.y as u16, RegSize::Word),
                    CpuReg::DP => (snem_core.cpu.dp as u16, RegSize::Word),
                    CpuReg::PC => (snem_core.cpu.pc as u16, RegSize::Word),
                    CpuReg::SP => (snem_core.cpu.sp as u16, RegSize::Word),
                };
                match reg_size {
                    RegSize::Byte => {
                        if msg.is_empty() {
                            format!("{:?} == {:02X}", reg, reg_val)
                        } else {
                            format!("{:?} == {:02X}: {}", reg, reg_val, msg)
                        }
                    }
                    RegSize::Word => {
                        if msg.is_empty() {
                            format!("{:?} == {:04X}", reg, reg_val)
                        } else {
                            format!("{:?} == {:04X}: {}", reg, reg_val, msg)
                        }
                    },
                }
            },
            LogKind::CpuFlag { flag, msg } => {
                let flag_val = match flag {
                    CpuFlag::C => snem_core.cpu.is_flag_set(scpu::Flag::FlagC),
                    CpuFlag::Z => snem_core.cpu.is_flag_set(scpu::Flag::FlagZ),
                    CpuFlag::I => snem_core.cpu.is_flag_set(scpu::Flag::FlagI),
                    CpuFlag::D => snem_core.cpu.is_flag_set(scpu::Flag::FlagD),
                    CpuFlag::X => snem_core.cpu.is_flag_set(scpu::Flag::FlagX),
                    CpuFlag::M => snem_core.cpu.is_flag_set(scpu::Flag::FlagM),
                    CpuFlag::V => snem_core.cpu.is_flag_set(scpu::Flag::FlagV),
                    CpuFlag::N => snem_core.cpu.is_flag_set(scpu::Flag::FlagN),
                    CpuFlag::Halted => snem_core.cpu.halted,
                    CpuFlag::Stopped => snem_core.cpu.stopped,
                    CpuFlag::Waiting => snem_core.cpu.waiting_for_interrupt,
                    CpuFlag::NMIPending => snem_core.cpu.nmi_pending,
                    CpuFlag::IRQPending => snem_core.cpu.irq_pending,
                };
                let flag_txt = if flag_val { "set" } else { "clear" };
                
                if msg.is_empty() {
                    format!("{:?} == {}", flag, flag_txt)
                } else {
                    format!("{:?} == {}: {}", flag, flag_txt, msg)
                }
            },
            LogKind::System { variable, msg } => {
                let value = match variable {
                    SystemVariable::Frame => snem_core.frame as usize,
                    SystemVariable::Dot => snem_core.ppu.dot as usize,
                    SystemVariable::Scanline => snem_core.ppu.scanline as usize,
                };
                
                if msg.is_empty() {
                    format!("{:?} == {}", variable, value)
                } else {
                    format!("{:?} == {}: {}", variable, value, msg)
                }
            },
            LogKind::Message { msg } => {
                format!("{}", msg)
            }
        }
    }
    
    pub fn label(&self) -> String {
        format!("Log\n{}", match self {
            LogKind::CpuReg { reg, .. } => format!("{:?}", reg),
            LogKind::CpuFlag { flag, .. } => format!("{:?}", flag),
            LogKind::System { variable, .. } => format!("{:?}", variable),
            LogKind::Message { .. } => "Message".to_string(),
        })
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
    /// Break indicator. Has 1 input, 0 outputs.
    Break { lit: bool },
    /// Log indicator. Has 1 input, 0 outputs.
    Log(LogKind)
}

impl NodeKind {
    pub fn input_count(&self) -> usize {
        match self {
            NodeKind::Condition { .. } => 0,
            NodeKind::And | NodeKind::Or => 2,
            NodeKind::Not => 1,
            NodeKind::Break { .. } => 1,
            NodeKind::Log { .. } => 1,
        }
    }

    pub fn output_count(&self) -> usize {
        match self {
            NodeKind::Condition { .. } => 1,
            NodeKind::And | NodeKind::Or | NodeKind::Not => 1,
            NodeKind::Break { .. } => 0,
            NodeKind::Log { .. } => 0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            NodeKind::Condition { .. } => "",
            NodeKind::And => "AND",
            NodeKind::Or => "OR",
            NodeKind::Not => "NOT",
            NodeKind::Break { .. } => "Break",
            NodeKind::Log { .. } => "",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            NodeKind::Condition { .. } => egui::Color32::from_rgb(60, 120, 200),
            NodeKind::And => egui::Color32::from_rgb(80, 160, 80),
            NodeKind::Or => egui::Color32::from_rgb(160, 120, 40),
            NodeKind::Not => egui::Color32::from_rgb(160, 60, 160),
            NodeKind::Break { .. } => egui::Color32::from_rgb(200, 60, 60),
            NodeKind::Log { .. } => egui::Color32::from_rgb(220, 220, 220),
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
                // Don't update changed nodes when evaluated by editor
                NodeKind::Condition(cond) => Some(cond.evaluate(snem_core)),
                NodeKind::And => Some(inputs.iter().all(|&b| b)),
                NodeKind::Or => Some(inputs.iter().any(|&b| b)),
                NodeKind::Not => Some(!inputs.first().copied().unwrap_or(false)),
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
    
    pub fn compile(&mut self, snem_core: &core::snemcore::Snemulator) -> CompiledGraph {
        // Index 0 is always a fallback 'false' for unconnected input ports.
        let mut ops = vec![FastOp::Constant(false)]; 
        let mut node_to_idx = HashMap::new();

        for id in self.topological_order() {
            let node = match self.nodes.get_mut(id) {
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

            let op = match &mut node.kind {
                NodeKind::Condition(cond) => {
                    match cond {
                        WatchpointKind::CpuReg { reg, cond: WatchpointCond::Changed(prev) } => {                            
                            *prev = reg.get_value(snem_core);
                        },
                        WatchpointKind::CpuFlag { flag, cond: WatchpointCondFlag::Changed(prev) } => {
                            *prev = flag.get_value(snem_core);
                        },
                        WatchpointKind::HardwareReg { reg, cond: WatchpointCond::Changed(prev) } => {
                            *prev = reg.get_value(snem_core);
                        }
                        WatchpointKind::HardwareFlag { flag, cond: WatchpointCondFlag::Changed(prev) } => {
                            *prev = flag.get_value(snem_core);
                        }
                        WatchpointKind::System { variable, cond: WatchpointCond::Changed(prev) } => {
                            *prev = variable.get_value(snem_core);
                        }
                        _ => {}
                    };
                    
                    FastOp::Cond(cond.clone())
                },
                NodeKind::And => FastOp::And(inputs[0], inputs[1]),
                NodeKind::Or  => FastOp::Or(inputs[0], inputs[1]),
                NodeKind::Not => FastOp::Not(inputs[0]),
                NodeKind::Break { .. } => FastOp::Output(inputs[0]),
                NodeKind::Log(kind) => FastOp::Log(inputs[0], kind.clone()),
                // _ => FastOp::Constant(false),
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
    Log(usize, LogKind)
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
                FastOp::Log(a, kind) => {
                    if results[*a] { log::debug!("{}", kind.message(snem_core)); }
                    false
                },
            };
        }
        break_triggered
    }
}