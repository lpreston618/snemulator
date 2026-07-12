#![allow(dead_code, static_mut_refs)]

// ---------------------------------------------------------------------------
// Test harness helpers
// ---------------------------------------------------------------------------

use crate::{cartridge::{Cartridge, MappingMode}, controller::ControllerData, debug::NullHarness, scpu::ioregs::CpuIoRegs, sppu::{Color, OAMSprite, regs::PpuRegs}, ssmp::ioports::ApuIoPorts, sysinfo::{CGRAM_SIZE, OAM_SIZE, OAM_SPRITE_COUNT, VRAM_SIZE, WRAM_SIZE}};

use crate::scpu::*;

static mut _FAKE_FBLANK_START_FLAG: bool = false;
static mut _FAKE_FBLANK_END_FLAG: bool = false;
static mut _FAKE_OPEN_BUS: u8 = 0;

/// Build a fresh CPU and a backing-store struct containing all the buffers
/// needed to construct a `CpuBus`. The reset vector is baked into the
/// blank cartridge so `cpu.reset()` finds it at 00:FFFC/FFFD.
pub(super) fn mk_cpu_and_backing(reset_vec: u16) -> (Cpu65c816, TestBacking) {
    (Cpu65c816::new(), TestBacking::new("TEST", reset_vec))
}

/// Backing store for a `CpuBus`.
pub(super) struct TestBacking {
    wram: Box<[u8; WRAM_SIZE]>,
    vram: Box<[u16; VRAM_SIZE]>,
    cgram: Box<[Color; CGRAM_SIZE]>,
    oam: Box<[OAMSprite; OAM_SPRITE_COUNT]>,
    raw_oam: Box<[u8; OAM_SIZE]>,
    ppu_regs: PpuRegs,
    cpu_regs: CpuIoRegs,
    apu_ports: ApuIoPorts,
    cart: Cartridge,
    controller_data: ControllerData,
}

impl TestBacking {
    pub(super) fn new(name: &str, reset_vec: u16) -> Self {
        Self {
            wram: Box::new([0u8; WRAM_SIZE]),
            vram: Box::new([0u16; VRAM_SIZE]),
            cgram: Box::new([Color::default(); CGRAM_SIZE]),
            oam: Box::new(std::array::repeat(OAMSprite::default())),
            raw_oam: Box::new([0u8; OAM_SIZE]),
            ppu_regs: PpuRegs::default(),
            cpu_regs: CpuIoRegs::default(),
            apu_ports: ApuIoPorts::default(),
            cart: Cartridge::test_blank(name, MappingMode::LoROM, reset_vec),
            controller_data: ControllerData::default(),
        }
    }

    pub(super) fn bus<'a>(&'a mut self, harness: &'a mut NullHarness) -> CpuBus<'a, NullHarness> {
        CpuBus {
            wram: &mut self.wram,
            vram: &mut self.vram,
            cgram: &mut self.cgram,
            oam: &mut self.oam,
            raw_oam: &mut self.raw_oam,
            ppu_regs: &mut self.ppu_regs,
            cpu_regs: &mut self.cpu_regs,
            apu_ports: &mut self.apu_ports,
            cart: &mut self.cart,
            dma: None,
            controller_data: &mut self.controller_data,
            harness,
            // SAFETY: Values are never read or accessed by NullProbe
            open_bus_value: unsafe { &mut _FAKE_OPEN_BUS },
            // fblank_start: unsafe { &mut _FAKE_FBLANK_START_FLAG },
            // fblank_end: unsafe { &mut _FAKE_FBLANK_END_FLAG },
        }
    }
}

/// Write a sequence of bytes into cartridge ROM using the force_write helper.
/// Used for opcode/operand setup since `read_prg` fetches from cartridge.
pub(super) fn write_rom(bus: &mut CpuBus<NullHarness>, mut addr: Address, bytes: &[u8]) {
    for &b in bytes {
        bus.cart.force_write(addr, b);
        addr.offset = addr.offset.wrapping_add(1);
    }
}

/// Write a sequence of bytes into RAM/MMIO via the CPU's bus path.
/// Used for data targets (WRAM, MMIO regions). Resets clocks afterward
/// so per-test timing assertions start from a clean baseline.
pub(super) fn write_ram<H: DebugHarness>(cpu: &mut Cpu65c816, bus: &mut CpuBus<H>, mut addr: Address, bytes: &[u8]) {
    for &b in bytes {
        cpu.write(bus, addr, b);
        addr.offset = addr.offset.wrapping_add(1);
    }
    cpu.clocks = 0;
}

/// Construct an Address from a bank/offset literal.
pub(super) fn addr(bank: u8, offset: u16) -> Address {
    Address { bank, offset }
}

/// Set the CPU's PC/PB so the next `execute()` will fetch from `(pb, pc)`.
pub(super) fn set_pc(cpu: &mut Cpu65c816, pb: u8, pc: u16) {
    cpu.pb = pb;
    cpu.pc = pc;
}