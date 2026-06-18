use anyhow::{anyhow, Result};
use cartridge::Cartridge;
use controller::{ControllerPlayer, JoypadButton, JoypadCmd, SnemController};
use dma::DmaController;
use scpu::bus::CpuBus;
use scpu::ioregs::CpuIoRegs;
use scpu::{Cpu65c816, CpuInterrupt};
use sppu::bus::PpuBus;
use sppu::color::Color;
use sppu::regs::PpuRegs;
use sppu::Ppu5C7x;
use ssmp::ioports::ApuIoPorts;
use ssmp::Ssmp;
use sysinfo::{CGRAM_SIZE, OAM_SIZE, VRAM_SIZE, WRAM_SIZE};
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::controller::ControllerData;
use crate::debug::DebugHarness;
use crate::sppu::VBLANK_START_SCANLINE;
use crate::sysinfo::CLOCKS_BETWEEN_AUTOREAD_STEPS;

pub mod cartridge;
pub mod controller;
pub mod dma;
pub mod scpu;
pub mod sppu;
pub mod ssmp;
pub mod sysinfo;
pub mod debug;
mod utils;

macro_rules! cpu_bus {
    ($core:ident, $harness:ident, $fblank_start:ident, $fblank_end:ident) => {
        CpuBus {
            wram: &mut $core.wram,
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            apu_ports: &mut $core.apu_ports,

            dma: Some(&mut $core.dma),

            controller_data: &mut $core.controller_data,
            cart: $core.cart.as_mut().unwrap(),

            harness: $harness,
            fblank_start: &mut $fblank_start,
            fblank_end: &mut $fblank_end,
        }
    };
}

macro_rules! dma_bus {
    ($core:ident, $harness:ident, $fblank_start:ident, $fblank_end:ident) => {
        CpuBus {
            wram: &mut $core.wram,
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            apu_ports: &mut $core.apu_ports,

            dma: None,

            controller_data: &mut $core.controller_data,
            cart: $core.cart.as_mut().unwrap(),

            harness: $harness,
            fblank_start: &mut $fblank_start,
            fblank_end: &mut $fblank_end,
        }
    };
}

macro_rules! ppu_bus {
    ($core:ident, $frame_buffer:ident, $harness:ident, $vblank_start:ident, $vblank_end:ident, $hblank_start:ident, $hblank_end:ident) => {
        PpuBus {
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            $frame_buffer,
            frame_ready: &mut $core.frame_ready,
            interrupt: &mut $core.cpu_interrupt,

            harness: $harness,
            vblank_start: &mut $vblank_start,
            vblank_end: &mut $vblank_end,
            hblank_start: &mut $hblank_start,
            hblank_end: &mut $hblank_end,
        }
    };
}

// Emulator core
pub struct Snemulator {
    p1_controller: SnemController,
    p2_controller: SnemController,

    pub cpu: Cpu65c816,
    pub ppu: Ppu5C7x,
    pub ssmp: Ssmp,

    pub wram: Box<[u8; WRAM_SIZE]>,
    pub vram: Box<[u16; VRAM_SIZE]>,
    pub cgram: Box<[Color; CGRAM_SIZE]>,
    pub oam: Box<[u8; OAM_SIZE]>,
    pub ppu_regs: PpuRegs,
    pub cpu_regs: CpuIoRegs,
    pub apu_ports: ApuIoPorts,

    pub dma: DmaController,

    pub controller_data: ControllerData,
    pub cpu_interrupt: Option<CpuInterrupt>,

    pub frame_ready: bool,

    pub cart: Option<Cartridge>,
    pub total_cycles: u64,
    pub frame: u64,

    clocks_last_frame: u64,

    random_seed: u64,
    rng: StdRng,
}

impl Snemulator {
    pub fn new() -> Self {
        let random_seed = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();

        Self {
            p1_controller: SnemController::new(),
            p2_controller: SnemController::new(),

            cpu: Cpu65c816::new(),
            ppu: Ppu5C7x::new(),
            ssmp: Ssmp::new(),

            wram: Box::new([0u8; WRAM_SIZE]),
            vram: Box::new([0u16; VRAM_SIZE]),
            cgram: Box::new([Color::BLACK; CGRAM_SIZE]),
            oam: Box::new([0u8; OAM_SIZE]),
            ppu_regs: PpuRegs::default(),
            cpu_regs: CpuIoRegs::default(),
            apu_ports: ApuIoPorts::default(),

            dma: DmaController::new(),

            controller_data: ControllerData::default(),
            cpu_interrupt: None,

            frame_ready: false,

            cart: None,
            total_cycles: 0u64,
            frame: 0u64,

            clocks_last_frame: 0,

            random_seed,
            rng: StdRng::seed_from_u64(random_seed),
        }
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.random_seed = seed;
        self.rng = StdRng::seed_from_u64(self.random_seed);
    }

    pub fn get_random_seed(&self) -> u64 {
        self.random_seed
    }

    pub fn power_on<H: DebugHarness>(&mut self, harness: &mut H) {
        if self.cart.is_none() {
            return;
        }

        self.clear_regs();

        self.wram.fill(0);
        self.vram.fill(0);
        self.cgram.fill(Color::BLACK);
        self.oam.fill(0);

        self.ppu_regs.power_on(&mut self.rng);
        self.cpu_regs.power_on();
        self.apu_ports.power_on();

        self.dma.power_on();

        let mut bus = cpu_bus!(self, harness, false, false);
        self.cpu.power_on(&mut bus);

        self.ssmp.power_on();
        self.ppu.power_on();

        if H::IS_DEBUGGING_HARNESS && H::TRACK_RESETS {
            harness.on_power(self);
        }

        if H::IS_DEBUGGING_HARNESS && H::TRACK_FBLANK {
            harness.on_fblank_start(self);
        }
    }

    pub fn reset<H: DebugHarness>(&mut self, harness: &mut H) {
        if self.cart.is_none() {
            return;
        }
        
        self.clear_regs();

        self.ppu_regs.reset();
        self.cpu_regs.reset();
        self.apu_ports.reset();

        self.dma.reset();

        let mut bus = cpu_bus!(self, harness, false, false);
        self.cpu.reset(&mut bus);

        self.ssmp.reset();
        self.ppu.reset();

        if H::IS_DEBUGGING_HARNESS && H::TRACK_RESETS {
            harness.on_reset(self);
        }
    }

    fn clear_regs(&mut self) {
        self.p1_controller = SnemController::new();
        self.p2_controller = SnemController::new();
        self.controller_data.joy1_latch = 0;
        self.controller_data.joy2_latch = 0;
        self.controller_data.joy1_data1_auto = 0;
        self.controller_data.joy2_data1_auto = 0;
        self.controller_data.joy1_data2_auto = 0;
        self.controller_data.joy2_data2_auto = 0;
        self.controller_data.joypad_cmd = None;
        self.cpu_interrupt = None;
        self.frame_ready = false;
        self.frame = 0;
        self.total_cycles = 0;
    }

    pub fn load_rom<H: DebugHarness>(&mut self, data: Vec<u8>, harness: &mut H) -> Result<()> {
        self.cart = Some(Cartridge::from_rom(data).map_err(|e| anyhow!(e))?);

        self.power_on(harness);

        Ok(())
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        match player {
            ControllerPlayer::Player1 => self.p1_controller.set_button(button, pressed),
            ControllerPlayer::Player2 => self.p2_controller.set_button(button, pressed),
        }
    }

    pub fn run_frame<H: DebugHarness>(
        &mut self,
        frame_buffer: &mut [u8],
        audio_buffer: &mut Vec<i16>,
        harness: &mut H,
    ) {
        self.frame_ready = false;

        self.clocks_last_frame = self.total_cycles;

        while !self.frame_ready {
            if H::IS_DEBUGGING_HARNESS && harness.should_stop(self) {
                return;
            }

            self.cycle(frame_buffer, audio_buffer, harness);
        }

        self.frame += 1;
    }

    fn cycle<H: DebugHarness>(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>, harness: &mut H) {
        let clocks = self.cpu.clocks.min(self.ppu.clocks);

        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        self.total_cycles += clocks as u64;

        if self.cpu.clocks == 0 {
            self.cycle_cpu(harness);
        }

        if self.ppu.clocks == 0 {
            self.cycle_ppu(frame_buffer, harness);
        }

        self.ssmp.cycle(clocks, audio_buffer, &mut self.apu_ports, harness);

        if self.cpu_regs.joypad_autoread_flag {
            if clocks >= self.controller_data.cycles_until_autoread {
                self.controller_data.cycles_until_autoread +=
                    CLOCKS_BETWEEN_AUTOREAD_STEPS - clocks;

                self.do_joypad_autoread_step();
            } else {
                self.controller_data.cycles_until_autoread -= clocks;
            }
        }

        if H::IS_DEBUGGING_HARNESS {
            harness.on_emulation_step(self);
        }
    }

    fn cycle_cpu<H: DebugHarness>(&mut self, harness: &mut H) {
        self.cpu.stopped = false;
        self.controller_data.joypad_cmd = None;

        let mut fblank_start_flag = false;
        let mut fblank_end_flag = false;

        if self.dma.hdma_needs_init && self.ppu.scanline == 0 {
            self.dma.hdma_needs_init = false;
            let mut bus = dma_bus!(self, harness, fblank_start_flag, fblank_end_flag);
            self.dma.hdma_init_channels(&mut bus);
        }

        if self.dma.hdma_en {
            self.cpu.stopped = true;
            let mut bus = dma_bus!(self, harness, fblank_start_flag, fblank_end_flag);
            self.dma.do_hdma(&mut bus, &mut self.cpu.stopped);
        }

        if !self.dma.hdma_en && self.dma.dma_en {
            self.cpu.stopped = true;
            let mut bus = dma_bus!(self, harness, fblank_start_flag, fblank_end_flag);
            self.dma.do_dma(&mut bus, &mut self.cpu.stopped);
        }

        let mut bus = cpu_bus!(self, harness, fblank_start_flag, fblank_end_flag);
        self.cpu.cycle(&mut bus);

        if H::IS_DEBUGGING_HARNESS && H::TRACK_FBLANK {
            if fblank_start_flag {
                harness.on_fblank_start(self);
            } else if fblank_end_flag {
                harness.on_fblank_end(self);
            }
        }

        match self.controller_data.joypad_cmd {
            Some(JoypadCmd::ClockJoy1) => self.controller_data.joy1_latch >>= 1,
            Some(JoypadCmd::ClockJoy2) => self.controller_data.joy2_latch >>= 1,
            _ => {}
        }

        if self.cpu_regs.latch_controllers {
            self.controller_data.joy1_latch = self.p1_controller.read_state();
            self.controller_data.joy2_latch = self.p2_controller.read_state();
        }

        if self.ppu.scanline == VBLANK_START_SCANLINE
            && self.cpu_regs.joypad_autoread_en
            && !self.cpu_regs.joypad_autoread_flag
        {
            self.controller_data.joypad_autoread_step = 0;
            self.cpu_regs.joypad_autoread_flag = true;
        }
    }

    fn cycle_ppu<H: DebugHarness>(&mut self, frame_buffer: &mut [u8], harness: &mut H) {
        self.cpu_interrupt = None;

        let mut vblank_start_flag = false;
        let mut vblank_end_flag = false;
        let mut hblank_start_flag = false;
        let mut hblank_end_flag = false;

        let mut bus = ppu_bus!(
            self, 
            frame_buffer, 
            harness, 
            vblank_start_flag, 
            vblank_end_flag, 
            hblank_start_flag, 
            hblank_end_flag
        );
        self.ppu.cycle(&mut bus);

        if H::IS_DEBUGGING_HARNESS && H::TRACK_PPU_STEP {
            harness.on_ppu_step(self);
        }

        if H::IS_DEBUGGING_HARNESS && H::TRACK_VBLANK {
            if vblank_start_flag {
                harness.on_vblank_start(self);
            } else if vblank_end_flag {
                harness.on_vblank_end(self);
            }
        }

        if H::IS_DEBUGGING_HARNESS && H::TRACK_HBLANK {
            if hblank_start_flag {
                harness.on_hblank_start(self);
            } else if hblank_end_flag {
                harness.on_hblank_end(self);
            }
        }

        match self.cpu_interrupt {
            Some(CpuInterrupt::IRQ) => self.cpu.irq_pending = true,
            Some(CpuInterrupt::NMI) => self.cpu.nmi_pending = true,
            _ => {}
        }

        if self.dma.hdma_pending
            && self.ppu.scanline < sppu::VBLANK_START_SCANLINE
            && self.ppu.dot == sppu::HBLANK_START_DOT
        {
            self.dma.hdma_en = self.dma.hdma_active_ch < 8;

            if self.dma.hdma_active_ch < 8 {
                self.dma.regs[self.dma.hdma_active_ch].hdma_do_transfer = true;
            }
        }
    }

    fn do_joypad_autoread_step(&mut self) {
        if self.controller_data.joypad_autoread_step < 12 {
            let button = match self.controller_data.joypad_autoread_step {
                0 => JoypadButton::B,
                1 => JoypadButton::Y,
                2 => JoypadButton::Select,
                3 => JoypadButton::Start,
                4 => JoypadButton::Up,
                5 => JoypadButton::Down,
                6 => JoypadButton::Left,
                7 => JoypadButton::Right,
                8 => JoypadButton::A,
                9 => JoypadButton::X,
                10 => JoypadButton::L1,
                11 => JoypadButton::R1,
                _ => unreachable!(),
            };

            self.controller_data.joy1_data1_auto <<= 1;
            self.controller_data.joy2_data1_auto <<= 1;
            // self.controller_data.joy1_data2_auto <<= 1;
            // self.controller_data.joy2_data2_auto <<= 1;

            self.controller_data.joy1_data1_auto |= if self.p1_controller.is_button_pressed(button)
            {
                1
            } else {
                0
            };
            self.controller_data.joy2_data1_auto |= if self.p2_controller.is_button_pressed(button)
            {
                1
            } else {
                0
            };
            // self.controller_data.joy1_data2_auto |=
            // self.controller_data.joy2_data2_auto |=
        } else {
            self.controller_data.joy1_data1_auto <<= 1;
            self.controller_data.joy2_data1_auto <<= 1;
        }

        self.controller_data.joypad_autoread_step += 1;

        if self.controller_data.joypad_autoread_step == 16 {
            self.controller_data.joypad_autoread_step = 0;
            self.cpu_regs.joypad_autoread_flag = false;
        }
    }

    // pub fn rom_slice(&self) -> &[u8] {
    //     self.cart.as_ref().unwrap().rom_slice()
    // }

    // pub fn sram_slice(&self) -> &[u8] {
    //     self.cart.as_ref().unwrap().sram_slice()
    // }
}

// impl Snemulator {
//     fn cycle_no_video(&mut self, audio_buffer: &mut Vec<i16>) {
//         let mut probe = self.probe.take().unwrap();

//         let clocks = self.cpu.clocks.min(self.ppu.clocks);

//         self.cpu.clocks -= clocks;
//         self.ppu.clocks -= clocks;
//         self.total_cycles += clocks as u64;

//         if self.cpu.clocks == 0 {
//             self.cycle_cpu();

//             probe.on_instruction(self);
//         }

//         if self.ppu.clocks == 0 {
//             let scanline = self.ppu.scanline;

//             self.cycle_ppu_no_output();

//             probe.on_dot(self);

//             if self.ppu.scanline != scanline {
//                 probe.on_scanline(self);
//             }
//         }

//         self.ssmp.cycle(clocks, audio_buffer, &mut self.apu_ports);

//         self.probe = Some(probe);
//     }

//     fn cycle_ppu_no_output(&mut self) {
//         self.cpu_interrupt = None;

//         let frame_buffer = &mut [];

//         let mut bus = ppu_bus!(self, frame_buffer);

//         self.ppu.cycle_no_output(&mut bus);

//         match self.cpu_interrupt {
//             Some(CpuInterrupt::IRQ) => {
//                 self.cpu.irq_pending = true;
//             }
//             Some(CpuInterrupt::NMI) => {
//                 self.cpu.nmi_pending = true;
//             }
//             _ => {}
//         }

//         if self.dma.hdma_pending
//             && self.ppu.scanline < sppu::VBLANK_START_SCANLINE
//             && self.ppu.dot == sppu::HBLANK_START_DOT
//         {
//             self.dma.hdma_en = true;
//             self.dma.regs[self.dma.hdma_active_ch].transfer_pattern_step = 0;
//         }
//     }
// }