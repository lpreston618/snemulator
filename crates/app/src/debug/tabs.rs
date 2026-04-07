mod cpu;
mod mem;
mod ppu;
mod watchpoints;

pub use cpu::CpuTab;
pub use ppu::PpuTab;
pub use mem::MemoryTab;
pub use watchpoints::WatchpointsTab;

#[derive(PartialEq, Clone, Copy)]
pub enum DebugTab { Cpu, Memory, Ppu, Watchpoints }

impl DebugTab {
    pub fn label(&self) -> &'static str {
        match self {
            DebugTab::Cpu         => "CPU",
            DebugTab::Memory      => "Memory",
            DebugTab::Ppu         => "PPU",
            DebugTab::Watchpoints => "Watchpoints",
        }
    }
}