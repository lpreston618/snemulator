mod chr;
mod cpu;
mod mem;
mod watchpoints;

pub use chr::ChrTab;
pub use cpu::CpuTab;
pub use mem::MemoryTab;
pub use watchpoints::WatchpointsTab;

#[derive(PartialEq, Clone, Copy)]
pub enum DebugTab { Cpu, Memory, ChrRam, Ppu, Watchpoints }

impl DebugTab {
    pub fn label(&self) -> &'static str {
        match self {
            DebugTab::Cpu         => "CPU",
            DebugTab::Memory      => "Memory",
            DebugTab::ChrRam      => "CHR RAM",
            DebugTab::Ppu         => "PPU",
            DebugTab::Watchpoints => "Watchpoints",
        }
    }
}