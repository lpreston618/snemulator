use crate::{ssmp::sdsp::{regs::SdspRegs, voices::VoiceRegs}, sysinfo::ARAM_SIZE};

pub struct SdspBus<'a> {
    pub aram: &'a mut [u8; ARAM_SIZE],
    pub sdsp_regs: &'a mut SdspRegs,
    pub voice_regs: &'a mut [VoiceRegs; 8],
}