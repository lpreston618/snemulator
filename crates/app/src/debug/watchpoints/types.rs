use std::cell::Cell;
use std::collections::HashMap;
use std::str::FromStr;

use snemcore::{Snemulator, scpu};

pub const HWVAL_NAMES: [&str; 10] = [
    // Regs
    "APUIO0", "APUIO1", "APUIO2", "APUIO3", "CPUIO0", "CPUIO1", "CPUIO2", "CPUIO3",
    // Flags
    "VBLANK", "FBLANK",
];

slotmap::new_key_type! { pub struct NodeId; }

pub trait WatchpointValueClone {
    fn clone_box(&self) -> Box<dyn WatchpointValue>;
}

pub trait WatchpointAsAnyMut {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait WatchpointAsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait WatchpointValue: WatchpointValueClone + WatchpointAsAnyMut + WatchpointAsAny {
    fn get_value(&self, snem_core: &Snemulator) -> usize;
    fn reg_size(&self) -> RegSize;
    fn label(&self) -> String;
    fn category(&self) -> RegCategory;
    fn value_str(&self, snem_core: &Snemulator) -> String {
        let val = self.get_value(snem_core);
        self.reg_size().format_val(val)
    }
}

impl<T: 'static + WatchpointValue + Clone> WatchpointAsAnyMut for T {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

impl<T: 'static + WatchpointValue + Clone> WatchpointAsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
}

impl<T: 'static + WatchpointValue + Clone> WatchpointValueClone for T {
    fn clone_box(&self) -> Box<dyn WatchpointValue> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn WatchpointValue> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RegCategory {
    CpuReg,
    CpuFlag,
    Ram,
    Vram,
    HwReg,
    HwFlag,
    SysInfo,
}

impl RegCategory {
    pub fn label(&self) -> &'static str {
        match self {
            RegCategory::CpuReg => "CPU Register",
            RegCategory::CpuFlag => "CPU Flag",
            RegCategory::Ram => "RAM",
            RegCategory::Vram => "VRAM",
            RegCategory::HwReg | RegCategory::HwFlag => "Hardware Register",
            RegCategory::SysInfo => "System Info",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RegSize {
    Bool,
    Byte,
    Word,
    Num,
}

impl RegSize {
    pub fn format_val(&self, val: usize) -> String {
        match self {
            RegSize::Bool => if val != 0 { "Set" } else { "Clear" }.to_string(),
            RegSize::Byte => format!("{:02X}", val),
            RegSize::Word => format!("{:04X}", val),
            RegSize::Num => format!("{}", val),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CpuReg {
    DB,
    PB,
    P,
    A,
    X,
    Y,
    DP,
    PC,
    SP,
}

impl WatchpointValue for CpuReg {
    fn get_value(&self, snem_core: &Snemulator) -> usize {
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

    fn label(&self) -> String {
        format!("{:?}", self)
    }

    fn reg_size(&self) -> RegSize {
        match self {
            CpuReg::DB | CpuReg::PB | CpuReg::P => RegSize::Byte,
            _ => RegSize::Word,
        }
    }

    fn category(&self) -> RegCategory {
        RegCategory::CpuReg
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CpuFlag {
    C,
    Z,
    I,
    D,
    X,
    M,
    V,
    N,
    Stopped,
    Halted,
    Waiting,
    NMIPending,
    IRQPending,
}

impl WatchpointValue for CpuFlag {
    fn get_value(&self, snem_core: &Snemulator) -> usize {
        let val = match self {
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
        };

        if val {
            1
        } else {
            0
        }
    }

    fn label(&self) -> String {
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
        }
        .to_string()
    }

    fn reg_size(&self) -> RegSize {
        RegSize::Bool
    }

    fn category(&self) -> RegCategory {
        RegCategory::CpuFlag
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

impl WatchpointValue for HardwareReg {
    fn get_value(&self, snem_core: &Snemulator) -> usize {
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

    fn label(&self) -> String {
        match self {
            HardwareReg::ApuIo0 => "APUIO0",
            HardwareReg::ApuIo1 => "APUIO1",
            HardwareReg::ApuIo2 => "APUIO2",
            HardwareReg::ApuIo3 => "APUIO3",
            HardwareReg::CpuIo0 => "CPUIO0",
            HardwareReg::CpuIo1 => "CPUIO1",
            HardwareReg::CpuIo2 => "CPUIO2",
            HardwareReg::CpuIo3 => "CPUIO3",
        }
        .to_string()
    }

    fn reg_size(&self) -> RegSize {
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

    fn category(&self) -> RegCategory {
        RegCategory::HwReg
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

impl WatchpointValue for HardwareFlag {
    fn get_value(&self, snem_core: &Snemulator) -> usize {
        let val = match self {
            HardwareFlag::VBlank => snem_core.cpu_regs.vblank_flag,
            HardwareFlag::FBlank => snem_core.ppu_regs.in_fblank,
        };

        if val {
            1
        } else {
            0
        }
    }

    fn label(&self) -> String {
        format!("{:?}", self)
    }

    fn reg_size(&self) -> RegSize {
        RegSize::Bool
    }

    fn category(&self) -> RegCategory {
        RegCategory::HwFlag
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SystemVariable {
    Frame,
    Dot,
    Scanline,
}

impl WatchpointValue for SystemVariable {
    fn get_value(&self, snem_core: &Snemulator) -> usize {
        match self {
            SystemVariable::Frame => snem_core.frame as usize,
            SystemVariable::Dot => snem_core.ppu.dot as usize,
            SystemVariable::Scanline => snem_core.ppu.scanline as usize,
        }
    }

    fn label(&self) -> String {
        format!("{:?}", self)
    }

    fn reg_size(&self) -> RegSize {
        RegSize::Num
    }

    fn category(&self) -> RegCategory {
        RegCategory::SysInfo
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WatchpointCond {
    Set,
    Clear,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    AndEqual,
    OrEqual,
    Changed,
}

impl WatchpointCond {
    pub fn label(&self, reg_size: RegSize, arg1: usize, arg2: usize) -> String {
        match reg_size {
            RegSize::Byte => match self {
                WatchpointCond::Equal => format!("== {:02X}", arg1),
                WatchpointCond::NotEqual => format!("!= {:02X}", arg1),
                WatchpointCond::GreaterThan => format!("> {:02X}", arg1),
                WatchpointCond::LessThan => format!("< {:02X}", arg1),
                WatchpointCond::AndEqual => format!("& {:02X}\n == {:02X}", arg1, arg2),
                WatchpointCond::OrEqual => format!("| {:02X}\n == {:02X}", arg1, arg2),
                WatchpointCond::Changed => "Changed".to_string(),
                _ => "".to_string(),
            },
            RegSize::Word => match self {
                WatchpointCond::Equal => format!("== {:04X}", arg1),
                WatchpointCond::NotEqual => format!("!= {:04X}", arg1),
                WatchpointCond::GreaterThan => format!("> {:04X}", arg1),
                WatchpointCond::LessThan => format!("< {:04X}", arg1),
                WatchpointCond::AndEqual => format!("& {:04X}\n == {:04X}", arg1, arg2),
                WatchpointCond::OrEqual => format!("| {:04X}\n == {:04X}", arg1, arg2),
                WatchpointCond::Changed => "Changed".to_string(),
                _ => "".to_string(),
            },
            RegSize::Num => match self {
                WatchpointCond::Equal => format!("== {}", arg1),
                WatchpointCond::NotEqual => format!("!= {}", arg1),
                WatchpointCond::GreaterThan => format!("> {}", arg1),
                WatchpointCond::LessThan => format!("< {}", arg1),
                WatchpointCond::AndEqual => format!("& {}\n == {}", arg1, arg2),
                WatchpointCond::OrEqual => format!("| {}\n == {}", arg1, arg2),
                WatchpointCond::Changed => "Changed".to_string(),
                _ => "".to_string(),
            },
            RegSize::Bool => match self {
                WatchpointCond::Set => "Set".to_string(),
                WatchpointCond::Clear => "Clear".to_string(),
                WatchpointCond::Changed => "Changed".to_string(),
                _ => "".to_string(),
            },
        }
    }

    pub fn dropdown_label(&self) -> &'static str {
        match self {
            WatchpointCond::Set => "Set",
            WatchpointCond::Clear => "Clear",
            WatchpointCond::Equal => "==",
            WatchpointCond::NotEqual => "!=",
            WatchpointCond::GreaterThan => ">",
            WatchpointCond::LessThan => "<",
            WatchpointCond::AndEqual => "&",
            WatchpointCond::OrEqual => "|",
            WatchpointCond::Changed => "Changed",
        }
    }

    pub fn evaluate(cond: &Self, val: usize, arg1: usize, arg2: usize) -> bool {
        match cond {
            WatchpointCond::Set => val != 0,
            WatchpointCond::Clear => val == 0,
            WatchpointCond::Equal => val == arg1,
            WatchpointCond::NotEqual => val != arg1,
            WatchpointCond::GreaterThan => val > arg1,
            WatchpointCond::LessThan => val < arg1,
            WatchpointCond::AndEqual => val & arg1 == arg2,
            WatchpointCond::OrEqual => val | arg1 == arg2,
            WatchpointCond::Changed => val != arg1,
        }
    }
}

#[derive(Clone)]
pub struct Watchpoint {
    pub val: Box<dyn WatchpointValue>,
    pub kind: RegCategory,
    pub cond: WatchpointCond,
    pub arg1: usize,
    pub arg2: usize,
    pub arg1_input_text: String,
    pub arg2_input_text: String,
    pub hw_reg_search: String,
}

impl Default for Watchpoint {
    fn default() -> Self {
        Self {
            val: Box::new(CpuReg::A),
            kind: RegCategory::CpuReg,
            cond: WatchpointCond::Equal,
            arg1: 0,
            arg2: 0,
            arg1_input_text: String::new(),
            arg2_input_text: String::new(),
            hw_reg_search: String::new(),
        }
    }
}

impl Watchpoint {
    pub fn evaluate(&self, snem_core: &Snemulator) -> bool {
        let val = self.val.get_value(snem_core);

        WatchpointCond::evaluate(&self.cond, val, self.arg1, self.arg2)
    }

    pub fn label(&self) -> String {
        format!(
            "{} {}",
            self.val.label(),
            self.cond.label(self.val.reg_size(), self.arg1, self.arg2)
        )
    }
}

#[derive(Clone, PartialEq)]
pub enum CounterMode {
    IncOnChange,
    IncOnTrue,
}

#[derive(Clone)]
pub struct Counter {
    pub fired: bool,
    pub prev: bool,
    pub count: usize,
    pub arg: usize,
    pub cond: WatchpointCond,
    pub reset_on_cond: bool,
    pub reset: usize,
    pub mode: CounterMode,
    pub input_text: String,
    pub reset_input_text: String,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            fired: false,
            prev: false,
            count: 0,
            arg: 100,
            cond: WatchpointCond::Equal,
            reset_on_cond: true,
            reset: 100,
            mode: CounterMode::IncOnTrue,
            input_text: String::new(),
            reset_input_text: String::new(),
        }
    }
}

impl Counter {
    pub fn evaluate(&self) -> bool {
        self.fired
    }

    pub fn label(&self) -> String {
        format!(
            "Counter\n{} {}",
            self.count,
            self.cond.label(RegSize::Num, self.arg, 0)
        )
    }
}

#[derive(Clone)]
pub struct Logpoint {
    pub regs: Vec<Box<dyn WatchpointValue>>,
    pub reg_types: Vec<RegCategory>,
    pub message_only: bool,
    pub msg: String,
    pub hw_reg_search_str: String,
}

impl Default for Logpoint {
    fn default() -> Self {
        Self {
            regs: Vec::new(),
            reg_types: Vec::new(),
            message_only: false,
            msg: String::new(),
            hw_reg_search_str: String::new(),
        }
    }
}

impl Logpoint {
    pub fn label(&self) -> String {
        if self.message_only {
            "Log".to_string()
        } else {
            match self.regs.len() {
                0 => "Log".to_string(),
                1 => format!("Log\n{}", self.regs[0].label()),
                2 => format!("Log\n{}, {}", self.regs[0].label(), self.regs[1].label()),
                _ => format!(
                    "Log\n{}, {}, ...",
                    self.regs[0].label(),
                    self.regs[1].label()
                ),
            }
        }
    }

    pub fn log_message(&self, snem_core: &Snemulator) {
        log::debug!(
            "{}",
            if self.message_only {
                self.msg.clone()
            } else {
                match self.regs.len() {
                    0 => {
                        if self.msg.is_empty() {
                            "No log values selected and no message".to_string()
                        } else {
                            self.msg.clone()
                        }
                    }
                    _ => {
                        let mut message = String::new();

                        for (i, reg) in self.regs.iter().enumerate() {
                            message += &format!("{} is {}", reg.label(), reg.value_str(snem_core));

                            if i != self.regs.len() - 1 {
                                message += ", "
                            }
                        }

                        if !self.msg.is_empty() {
                            message += &format!(": {}", self.msg);
                        }

                        message
                    }
                }
            }
        );
    }
}

pub enum FastOp {
    Constant(bool),
    // Need a special case for changed watchpoints
    CondChanged {
        prev_value: Cell<usize>,
        value: Box<dyn WatchpointValue>,
        id: NodeId,
    },
    Cond(Watchpoint),
    CounterRisingEdge {
        input: usize,
        prev: Cell<bool>,
        count: Cell<usize>,
        arg: usize,
        reset: usize,
        cond: WatchpointCond,
        fired: Cell<bool>,
        id: NodeId,
    },
    CounterHigh {
        input: usize,
        count: Cell<usize>,
        arg: usize,
        reset: usize,
        cond: WatchpointCond,
        fired: Cell<bool>,
        id: NodeId,
    },
    And(usize, usize),
    Or(usize, usize),
    Not(usize),
    Output(usize),
    Log(usize, Logpoint),
}

#[derive(Default)]
pub struct CompiledGraph {
    ops: Vec<FastOp>,
}

impl CompiledGraph {
    pub fn new(ops: Vec<FastOp>) -> Self {
        Self {
            ops
        }
    }
    
    pub fn evaluate(&self, snem_core: &Snemulator) -> bool {
        if self.ops.is_empty() {
            return false;
        }

        let mut results = vec![false; self.ops.len()];
        let mut break_triggered = false;

        for (i, op) in self.iter().enumerate() {
            results[i] = match op {
                FastOp::Constant(val) => val.clone(),
                FastOp::CondChanged {
                    prev_value, value, ..
                } => {
                    let val = value.get_value(snem_core);
                    let prev = prev_value.replace(val);
                    val != prev
                }
                FastOp::Cond(wp) => wp.evaluate(snem_core),
                FastOp::And(a, b) => results[*a] && results[*b],
                FastOp::Or(a, b) => results[*a] || results[*b],
                FastOp::Not(a) => !results[*a],
                FastOp::CounterRisingEdge {
                    input,
                    prev,
                    count,
                    arg,
                    cond,
                    reset,
                    fired,
                    ..
                } => {
                    let val = results[*input];
                    let prev_val = prev.replace(val);

                    if val && !prev_val {
                        count.replace(count.get() + 1);
                    }

                    fired.set(WatchpointCond::evaluate(cond, count.get(), *arg, 0));

                    if count.get() == *reset {
                        count.set(0);
                    }

                    fired.get()
                }
                FastOp::CounterHigh {
                    input,
                    count,
                    arg,
                    cond,
                    reset,
                    fired,
                    ..
                } => {
                    if results[*input] {
                        count.replace(count.get() + 1);
                    }

                    fired.set(WatchpointCond::evaluate(cond, count.get(), *arg, 0));

                    if count.get() == *reset {
                        count.set(0);
                    }

                    fired.get()
                }
                FastOp::Output(a) => {
                    if results[*a] {
                        break_triggered = true;
                    }
                    false
                }
                FastOp::Log(a, lp) => {
                    if results[*a] {
                        lp.log_message(snem_core);
                    }
                    false
                }
            };
        }
        break_triggered
    }

    pub fn iter(&self) -> impl Iterator<Item = &FastOp> {
        self.ops.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut FastOp> {
        self.ops.iter_mut()
    }
}


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