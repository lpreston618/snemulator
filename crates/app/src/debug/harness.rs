use std::collections::HashSet;

use snemcore::debug::DebugHarness;

use crate::debug::{stack_tracker::StackTracker, tabs::ssmp::RingBuffer};

const JSL_OPCODE: u8 = 0x22;
const JSR_OPCODE: u8 = 0x20;
const JSR_INDIRECT_OPCODE: u8 = 0xFC;

// const RTI_OPCODE: u8 = 0x40;
const RTS_OPCODE: u8 = 0x60;
const RTL_OPCODE: u8 = 0x68;

pub const DSP_SAMPLE_RATE: usize = 32_000;
pub const SAMPLE_HISTORY_SECONDS: f32 = 10.0;
pub const SAMPLE_HISTORY_LEN: usize = (SAMPLE_HISTORY_SECONDS as usize) * DSP_SAMPLE_RATE;

pub const ENVELOPE_HISTORY_LEN: usize = 256;

#[derive(Clone, Copy)]
pub enum StopCondition {
    AnyScpuCycle,
    ScpuInstruction,
    Interrupt,
    Frame,
    StepOverSubroutine { depth: usize },
    DmaStart { ch: u8 },
    DmaEnd { ch: u8 },
    HdmaStart { ch: u8 },
    HdmaEnd { ch: u8 },
    SampleGenerated,
    SpcInstruction,
    KeyOn { v: u8 },
    KeyOff { v: u8 },
}

pub struct MainDebugHarness {
    pub stop_condition: Option<StopCondition>,
    pub stop_emulation: bool,

    pub breakpoints: HashSet<u32>,

    pub stack_tracker: StackTracker,

    pub voices_just_keyed_on: [bool; 8],
    pub voice_buffers: [(RingBuffer<SAMPLE_HISTORY_LEN>, RingBuffer<SAMPLE_HISTORY_LEN>); 8],
    pub mix_buffers: (RingBuffer<SAMPLE_HISTORY_LEN>, RingBuffer<SAMPLE_HISTORY_LEN>),

    /// Per-voice ring buffer of recent envelope values for the live ADSR painter.
    pub envelope_history: [RingBuffer<ENVELOPE_HISTORY_LEN>; 8],
}

impl MainDebugHarness {
    pub fn new() -> Self {
        Self {
            stop_condition: None,
            stop_emulation: false,
            breakpoints: HashSet::new(),
            stack_tracker: StackTracker::new(),
            voices_just_keyed_on: [false; 8],
            voice_buffers: std::array::from_fn(|_| (RingBuffer::new(), RingBuffer::new())),
            envelope_history: std::array::from_fn(|_| RingBuffer::new()),
            mix_buffers: (RingBuffer::new(), RingBuffer::new()),
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
        if matches!(self.stop_condition, Some(StopCondition::AnyScpuCycle)) {
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

    fn on_voice_key_on(&mut self, _voice_regs: &mut snemcore::ssmp::sdsp::voices::VoiceRegs, voice: usize) {
        self.voices_just_keyed_on[voice] = true;

        match self.stop_condition {
            Some(StopCondition::KeyOn { v }) => {
                self.stop_emulation |= voice == v as usize;
            }
            _ => {}
        }
    }

    fn on_voice_key_off(&mut self, _voice_regs: &mut snemcore::ssmp::sdsp::voices::VoiceRegs, voice: usize) {
        self.voices_just_keyed_on[voice] = false;

        match self.stop_condition {
            Some(StopCondition::KeyOff { v }) => {
                self.stop_emulation |= voice == v as usize;
            }
            _ => {}
        }
    }

    fn on_sample_generated(&mut self, ssmp: &mut snemcore::ssmp::Ssmp) {
        if matches!(self.stop_condition, Some(StopCondition::SampleGenerated)) {
            self.stop_emulation = true;
        }

        let left_sample  = ssmp.sdsp.last_generated_left;
        let right_sample = ssmp.sdsp.last_generated_right;

        self.mix_buffers.0.push(left_sample);
        self.mix_buffers.1.push(right_sample);

        for voice in 0..8usize {
            let left_sample  = ssmp.voice_regs[voice].last_generated_left;
            let right_sample = ssmp.voice_regs[voice].last_generated_right;

            self.voice_buffers[voice].0.push(left_sample);
            self.voice_buffers[voice].1.push(right_sample);
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
                StopCondition::ScpuInstruction | StopCondition::AnyScpuCycle => self.stop_emulation = true,
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

    fn on_vblank_start(&mut self, core: &mut snemcore::Snemulator) {
        for voice in 0..8usize {
            self.envelope_history[voice].push(core.ssmp.voice_regs[voice].envelope as i16);
        }

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