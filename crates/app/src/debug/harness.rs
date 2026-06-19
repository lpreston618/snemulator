use std::collections::HashSet;

use snemcore::debug::DebugHarness;

use crate::debug::stack_tracker::StackTracker;

const JSL_OPCODE: u8 = 0x22;
const JSR_OPCODE: u8 = 0x20;
const JSR_INDIRECT_OPCODE: u8 = 0xFC;

// const RTI_OPCODE: u8 = 0x40;
const RTS_OPCODE: u8 = 0x60;
const RTL_OPCODE: u8 = 0x68;




#[derive(Clone, Copy)]
pub enum StopCondition {
    AnyCpuCycle,
    Instruction,
    Interrupt,
    Frame,
    StepOverSubroutine { depth: usize },
    DmaStart { ch: u8 },
    DmaEnd { ch: u8 },
    HdmaStart { ch: u8 },
    HdmaEnd { ch: u8 },
}

pub struct MainDebugHarness {
    pub stop_condition: Option<StopCondition>,
    pub stop_emulation: bool,

    pub breakpoints: HashSet<u32>,

    pub stack_tracker: StackTracker,
}

impl MainDebugHarness {
    pub fn new() -> Self {
        Self {
            stop_condition: None,
            stop_emulation: false,
            breakpoints: HashSet::new(),
            stack_tracker: StackTracker::new(),
        }
    }
}

impl DebugHarness for MainDebugHarness {
    const IS_DEBUGGING_HARNESS: bool = true;

    fn should_stop(&mut self, _core: &mut snemcore::Snemulator) -> bool {
        self.stop_emulation
    }

    fn on_dma_transfer(
        &mut self,
        _dma: &mut snemcore::dma::DmaController,
        _channel: usize,
        _src_addr: snemcore::scpu::Address,
        _dst_addr: snemcore::scpu::Address,
        _value: u8
    ) {
        if matches!(self.stop_condition, Some(StopCondition::AnyCpuCycle)) {
            self.stop_emulation = true;
        }
    }

    fn on_dma_start(&mut self, _dma: &mut snemcore::dma::DmaController, channel: usize) {
        match self.stop_condition {
            Some(StopCondition::DmaStart { ch }) => {
                self.stop_emulation |= channel == ch as usize;
            }
            _ => {}
        }
    }

    fn on_dma_end(&mut self, _dma: &mut snemcore::dma::DmaController, channel: usize) {
        match self.stop_condition {
            Some(StopCondition::DmaEnd { ch }) => {
                self.stop_emulation |= channel == ch as usize;
            }
            _ => {}
        }
    }

    // fn on_hdma_transfer(
    //     &mut self,
    //     _dma: &mut snemcore::dma::DmaController,
    //     _channel: usize,
    //     _src_addr: snemcore::scpu::Address,
    //     _dst_addr: snemcore::scpu::Address,
    //     _value: u8
    // ) {
    //     if matches!(self.stop_condition, Some(StopCondition::AnyCpuCycle)) {
    //         self.stop_emulation = true;
    //     }
    // }

    fn on_instruction(&mut self, cpu: &mut snemcore::scpu::Cpu65c816, prg_bytes: &[u8]) {
        self.stack_tracker.on_instruction(cpu, prg_bytes);

        if let Some(stop_cond) = self.stop_condition {
            match stop_cond {
                StopCondition::Instruction | StopCondition::AnyCpuCycle => self.stop_emulation = true,
                StopCondition::StepOverSubroutine { depth } => {
                    let opcode = prg_bytes[0];

                    if opcode == JSL_OPCODE || opcode == JSR_OPCODE || opcode == JSR_INDIRECT_OPCODE {
                        self.stop_condition = Some(StopCondition::StepOverSubroutine { depth: depth + 1 });
                    } else if opcode == RTL_OPCODE || opcode == RTS_OPCODE {
                        // If step over clicked on a return instruction, depth will be 0 and instr will be return.
                        if depth <= 1 {
                            self.stop_emulation = true;
                        }

                        self.stop_condition = Some(StopCondition::StepOverSubroutine { depth: depth - 1 });
                    } else {
                        self.stop_emulation = depth == 0;
                    }
                }
                _ => {}
            }
        }

        let full_pc = (cpu.pb as u32) << 16 | cpu.pc as u32;
        if self.breakpoints.contains(&full_pc) {
            self.stop_emulation = true;
        }
    }

    fn on_vblank_start(&mut self, _core: &mut snemcore::Snemulator) {
        if matches!(self.stop_condition, Some(StopCondition::Frame)) {
            self.stop_emulation = true;
        }
    }

    fn on_interrupt(&mut self, cpu: &mut snemcore::scpu::Cpu65c816, kind: snemcore::scpu::CpuInterrupt) {
        self.stack_tracker.on_interrupt(cpu, kind);

        if matches!(self.stop_condition, Some(StopCondition::Interrupt)) {
            self.stop_emulation = true;
        }
    }

    fn on_stack_push(&mut self, cpu: &mut snemcore::scpu::Cpu65c816, value: u8) {
        self.stack_tracker.on_stack_push(cpu, value);
    }

    fn on_stack_pop(&mut self, cpu: &mut snemcore::scpu::Cpu65c816, value: u8) {
        self.stack_tracker.on_stack_pop(cpu, value);
    }

    fn on_power(&mut self, _core: &mut snemcore::Snemulator) {
        self.stack_tracker.clear();
    }

    fn on_reset(&mut self, _core: &mut snemcore::Snemulator) {
        self.stack_tracker.clear();
    }
}