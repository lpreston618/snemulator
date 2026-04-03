use std::cell::Cell;
use std::collections::HashMap;
use std::str::FromStr;

use crate::Snemulator;
use crate::scpu;

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
