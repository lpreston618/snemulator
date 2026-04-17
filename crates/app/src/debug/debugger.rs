use std::collections::HashSet;
use anyhow::Result;

use snemcore::Snemulator;
use snemcore::probe::DebugProbe;
use snemcore::sppu::ColorLayer;
use snemcore::sysinfo::{SCREEN_WIDTH, SCREEN_HEIGHT};

use crate::debug::breakpoints::BreakpointInfo;
use crate::debug::watchpoints::engine::{CallbackType, WatchpointAction, WatchpointEngine};

const LAYER_BUFFER_SIZE: usize = 4 * ((SCREEN_WIDTH / 2) * (SCREEN_HEIGHT / 2)) as usize;

pub struct LayerBuffers {
    pub bg1: Vec<u8>,
    pub bg2: Vec<u8>,
    pub bg3: Vec<u8>,
    pub bg4: Vec<u8>,
    pub obj: Vec<u8>,
}

impl LayerBuffers {
    pub fn new() -> Self {
        Self {
            bg1: vec![0u8; LAYER_BUFFER_SIZE],
            bg2: vec![0u8; LAYER_BUFFER_SIZE],
            bg3: vec![0u8; LAYER_BUFFER_SIZE],
            bg4: vec![0u8; LAYER_BUFFER_SIZE],
            obj: vec![0u8; LAYER_BUFFER_SIZE],
        }
    }
    
    fn clear_all(&mut self) {
        self.clear_layer(ColorLayer::Bg1);
        self.clear_layer(ColorLayer::Bg2);
        self.clear_layer(ColorLayer::Bg3);
        self.clear_layer(ColorLayer::Bg4);
        self.clear_layer(ColorLayer::Obj);
    }

    fn clear_layer(&mut self, layer: ColorLayer) {
        let layer_buffer = match layer {
            ColorLayer::Bg1 => &mut self.bg1[..],
            ColorLayer::Bg2 => &mut self.bg2[..],
            ColorLayer::Bg3 => &mut self.bg3[..],
            ColorLayer::Bg4 => &mut self.bg4[..],
            ColorLayer::Obj => &mut self.obj[..],
            _ => return,
        };
        
        layer_buffer.chunks_mut(4).enumerate().for_each(|(i, p)| {
            let y = i / 256;
            let x = i % 256;

            p.copy_from_slice(if (x / 2 + y / 2) % 2 == 0 {
                &[0x50, 0x50, 0x50, 255]
            } else {
                &[0x30, 0x30, 0x30, 255]
            });
        });
    }
}

pub struct Debugger {
    pub breakpoints: HashSet<BreakpointInfo>,
    pub layer_buffers: LayerBuffers,
    pub breakpoint_hit: bool,
    pub update_textures: bool,
    pub watchpoint_hit: bool,
    pub wp_engine: WatchpointEngine,
    pub should_stop: bool,
}

impl DebugProbe for Debugger {
    fn init(&mut self, core: &mut Snemulator<Self>) {
        match self.wp_engine.init(core) {
            Err(e) => log::error!("Failed to initialize watchpoint engine: {}", e),
            _ => {}
        }
    }
    
    fn should_stop(&mut self) -> bool {
        self.should_stop
    }
    
    fn resume_emulation(&mut self) {
        self.breakpoint_hit = false;
        self.watchpoint_hit = false;
        self.should_stop = false;
    }
    
    fn on_emulation_cycle(&mut self, core: &mut Snemulator<Self>) {
        match self.wp_engine.execute_callback(CallbackType::EmulationCycle, ()) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_dot(&mut self, core: &mut Snemulator<Self>) {
        if self.update_textures {
            if core.ppu.x == 0 && core.ppu.y == 0 {
                self.layer_buffers.clear_all();
            }
            
            if core.ppu.x < 256 && core.ppu.y < 224 {
                core.update_layer_buffers(
                    &mut self.layer_buffers.bg1[..],
                    &mut self.layer_buffers.bg2[..],
                    &mut self.layer_buffers.bg3[..],
                    &mut self.layer_buffers.bg4[..],
                    &mut self.layer_buffers.obj[..],
                );
            }
        }
        
        match self.wp_engine.execute_callback(CallbackType::Dot, ()) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_scanline(&mut self, core: &mut Snemulator<Self>) {
        match self.wp_engine.execute_callback(CallbackType::Scanline, ()) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_frame(&mut self, core: &mut Snemulator<Self>) {
        match self.wp_engine.execute_callback(CallbackType::Frame, ()) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_instruction(&mut self, core: &mut Snemulator<Debugger>) {
        let full_pc = (core.cpu.pb as u32) << 16 | core.cpu.pc as u32;
        
        if self.breakpoints.contains(&BreakpointInfo::new(full_pc)) {
            self.hit_breakpoint();
        }
        
        match self.wp_engine.execute_callback(CallbackType::Instruction, ()) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_interrupt(&mut self, kind: snemcore::scpu::CpuInterrupt) {
        match self.wp_engine.execute_callback(CallbackType::Interrupt, kind as u8) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_memory_read(&mut self, addr: u32, value: u8) {
        match self.wp_engine.execute_callback(CallbackType::MemoryRead, (addr, value)) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_memory_write(&mut self, addr: u32, value: u8) {
        match self.wp_engine.execute_callback(CallbackType::MemoryWrite, (addr, value)) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }

    fn on_dma_start(&mut self, channel: usize) {
        match self.wp_engine.execute_callback(CallbackType::DmaStart, channel) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_dma_transfer(&mut self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) {
        match self.wp_engine.execute_callback(CallbackType::DmaTransfer, (channel, src_addr, dst_addr, value)) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }

    fn on_dma_end(&mut self, channel: usize) {
        match self.wp_engine.execute_callback(CallbackType::DmaEnd, channel) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_hdma_start(&mut self, channel: usize) {
        match self.wp_engine.execute_callback(CallbackType::HdmaStart, channel) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
    
    fn on_hdma_transfer(&mut self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) {
        match self.wp_engine.execute_callback(CallbackType::HdmaTransfer, (channel, src_addr, dst_addr, value)) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }

    fn on_hdma_end(&mut self, channel: usize) {
        match self.wp_engine.execute_callback(CallbackType::HdmaEnd, channel) {
            WatchpointAction::Break => { self.hit_watchpoint(); },
            _ => {}
        }
    }
}

impl Debugger {
    pub fn new() -> Result<Self> {
        Ok(Self {
            breakpoints: HashSet::new(),
            layer_buffers: LayerBuffers::new(),
            breakpoint_hit: false,
            watchpoint_hit: false,
            update_textures: true,
            wp_engine: WatchpointEngine::new(),
            should_stop: false,
        })
    }
    
    fn hit_watchpoint(&mut self) {
        self.watchpoint_hit = true;
        self.should_stop = true;
    }
    
    fn hit_breakpoint(&mut self) {
        self.breakpoint_hit = true;
        self.should_stop = true;
    }
}
