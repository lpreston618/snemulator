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
use crate::sppu::VBLANK_START_SCANLINE;
use crate::sysinfo::CLOCKS_BETWEEN_AUTOREAD_STEPS;

pub mod cartridge;
pub mod controller;
pub mod dma;
pub mod scpu;
pub mod sppu;
pub mod ssmp;
pub mod sysinfo;
mod utils;

macro_rules! cpu_bus {
    ($core:ident) => {
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
        }
    };
}

macro_rules! dma_bus {
    ($core:ident) => {
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
        }
    };
}

macro_rules! ppu_bus {
    ($core:ident, $frame_buffer:ident) => {
        PpuBus {
            vram: &mut $core.vram,
            cgram: &mut $core.cgram,
            oam: &mut $core.oam,
            ppu_regs: &mut $core.ppu_regs,
            cpu_regs: &mut $core.cpu_regs,
            $frame_buffer,
            frame_ready: &mut $core.frame_ready,
            interrupt: &mut $core.cpu_interrupt,
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

    pub fn power_on(&mut self) {
        self.clear_regs();

        self.wram.fill(0);
        self.vram.fill(0);
        self.cgram.fill(Color::BLACK);
        self.oam.fill(0);

        self.ppu_regs.power_on(&mut self.rng);
        self.cpu_regs.power_on();
        self.apu_ports.power_on();

        self.dma.power_on();

        let mut bus = cpu_bus!(self);
        self.cpu.power_on(&mut bus);

        self.ssmp.power_on();
        self.ppu.power_on();
    }

    pub fn reset(&mut self) {
        self.clear_regs();

        self.ppu_regs.reset();
        self.cpu_regs.reset();
        self.apu_ports.reset();

        self.dma.reset();

        let mut bus = cpu_bus!(self);
        self.cpu.reset(&mut bus);

        self.ssmp.reset();
        self.ppu.reset();
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

    pub fn load_rom(&mut self, data: Vec<u8>) -> Result<()> {
        self.cart = Some(Cartridge::from_rom(data).map_err(|e| anyhow!(e))?);

        self.power_on();

        Ok(())
    }

    pub fn set_button(&mut self, player: ControllerPlayer, button: JoypadButton, pressed: bool) {
        match player {
            ControllerPlayer::Player1 => self.p1_controller.set_button(button, pressed),
            ControllerPlayer::Player2 => self.p2_controller.set_button(button, pressed),
        }
    }

    pub fn run_frame(
        &mut self,
        frame_buffer: &mut [u8],
        audio_buffer: &mut Vec<i16>,
    ) {
        self.frame_ready = false;

        self.clocks_last_frame = self.total_cycles;

        while !self.frame_ready {
            self.cycle(frame_buffer, audio_buffer);
        }

        log::debug!("Clocks this frame: {}", self.total_cycles - self.clocks_last_frame);

        self.frame += 1;
    }

    fn cycle(&mut self, frame_buffer: &mut [u8], audio_buffer: &mut Vec<i16>) {
        let clocks = self.cpu.clocks.min(self.ppu.clocks);

        self.cpu.clocks -= clocks;
        self.ppu.clocks -= clocks;
        self.total_cycles += clocks as u64;

        if self.cpu.clocks == 0 {
            self.cycle_cpu();
        }

        if self.ppu.clocks == 0 {
            self.cycle_ppu(frame_buffer);
        }

        self.ssmp.cycle(clocks, audio_buffer, &mut self.apu_ports);

        if self.cpu_regs.joypad_autoread_flag {
            if clocks >= self.controller_data.cycles_until_autoread {
                self.controller_data.cycles_until_autoread +=
                    CLOCKS_BETWEEN_AUTOREAD_STEPS - clocks;

                self.do_joypad_autoread_step();
            } else {
                self.controller_data.cycles_until_autoread -= clocks;
            }
        }
    }

    fn cycle_cpu(&mut self) {
        self.cpu.stopped = false;
        self.controller_data.joypad_cmd = None;

        if self.dma.hdma_needs_init && self.ppu.scanline == 0 {
            self.dma.hdma_needs_init = false;
            let mut bus = dma_bus!(self);
            self.dma.hdma_init_channels(&mut bus);
        }

        if self.dma.hdma_en {
            self.cpu.stopped = true;
            let mut bus = dma_bus!(self);
            self.dma.do_hdma(&mut bus, &mut self.cpu.stopped);
        }

        if !self.dma.hdma_en && self.dma.dma_en {
            self.cpu.stopped = true;
            let mut bus = dma_bus!(self);
            self.dma.do_dma(&mut bus, &mut self.cpu.stopped);
        }

        let mut bus = cpu_bus!(self);
        self.cpu.cycle(&mut bus);

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

    fn cycle_ppu(&mut self, frame_buffer: &mut [u8]) {
        self.cpu_interrupt = None;

        let mut bus = ppu_bus!(self, frame_buffer);
        self.ppu.cycle(&mut bus);

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