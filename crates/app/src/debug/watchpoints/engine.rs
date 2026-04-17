use anyhow::Result;

use std::marker::PhantomData;
use std::any::Any;

use crate::debug::{debugger::{DebugControl, Debugger}, watchpoints::{interface::ControlInterface, luainclude::LUA_INCLUDE}};
use mlua::{AnyUserData, Function, IntoLuaMulti, Lua, UserData};
use snemcore::{Snemulator, probe::DebugProbe, scpu::CpuInterrupt};

use crate::debug::watchpoints::interface::SnemulatorInterface;

#[derive(Clone, Copy)]
pub enum WatchpointAction {
    Continue,
    Break,
}

impl UserData for WatchpointAction {}

struct Callbacks {
    on_emulation_cycle: Option<Function>,
    on_dot: Option<Function>,
    on_scanline: Option<Function>,
    on_frame: Option<Function>,
    on_instruction: Option<Function>,
    on_interrupt: Option<Function>,
    on_memory_read: Option<Function>,
    on_memory_write: Option<Function>,
    on_dma_start: Option<Function>,
    on_dma_transfer: Option<Function>,
    on_dma_end: Option<Function>,
    on_hdma_start: Option<Function>,
    on_hdma_transfer: Option<Function>,
    on_hdma_end: Option<Function>,
}

impl Callbacks {
    fn get(&self, callback: CallbackType) -> Option<&Function> {
        match callback {
            CallbackType::EmulationCycle => self.on_emulation_cycle.as_ref(),
            CallbackType::Dot => self.on_dot.as_ref(),
            CallbackType::Scanline => self.on_scanline.as_ref(),
            CallbackType::Frame => self.on_frame.as_ref(),
            CallbackType::Instruction => self.on_instruction.as_ref(),
            CallbackType::Interrupt => self.on_interrupt.as_ref(),
            CallbackType::MemoryRead => self.on_memory_read.as_ref(),
            CallbackType::MemoryWrite => self.on_memory_write.as_ref(),
            CallbackType::DmaStart => self.on_dma_start.as_ref(),
            CallbackType::DmaTransfer => self.on_dma_transfer.as_ref(),
            CallbackType::DmaEnd => self.on_dma_end.as_ref(),
            CallbackType::HdmaStart => self.on_hdma_start.as_ref(),
            CallbackType::HdmaTransfer => self.on_hdma_transfer.as_ref(),
            CallbackType::HdmaEnd => self.on_hdma_end.as_ref(),
        }
    }
    
    fn set(&mut self, callback: CallbackType, func: Option<Function>) {
        match callback {
            CallbackType::EmulationCycle => self.on_emulation_cycle = func,
            CallbackType::Dot => self.on_dot = func,
            CallbackType::Scanline => self.on_scanline = func,
            CallbackType::Frame => self.on_frame = func,
            CallbackType::Instruction => self.on_instruction = func,
            CallbackType::Interrupt => self.on_interrupt = func,
            CallbackType::MemoryRead => self.on_memory_read = func,
            CallbackType::MemoryWrite => self.on_memory_write = func,
            CallbackType::DmaStart => self.on_dma_start = func,
            CallbackType::DmaTransfer => self.on_dma_transfer = func,
            CallbackType::DmaEnd => self.on_dma_end = func,
            CallbackType::HdmaStart => self.on_hdma_start = func,
            CallbackType::HdmaTransfer => self.on_hdma_transfer = func,
            CallbackType::HdmaEnd => self.on_hdma_end = func,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CallbackType {
    EmulationCycle,
    Dot,
    Scanline,
    Frame,
    Instruction,
    Interrupt,
    MemoryRead,
    MemoryWrite,
    DmaStart,
    DmaTransfer,
    DmaEnd,
    HdmaStart,
    HdmaTransfer,
    HdmaEnd,
}

impl CallbackType {
    const ALL: &'static [Self] = &[
        Self::EmulationCycle,
        Self::Dot,
        Self::Scanline,
        Self::Frame,
        Self::Instruction,
        Self::Interrupt,
        Self::MemoryRead,
        Self::MemoryWrite,
        Self::DmaStart,
        Self::DmaTransfer,
        Self::DmaEnd,
        Self::HdmaStart,
        Self::HdmaTransfer,
        Self::HdmaEnd,
    ];
    
    fn lua_fn_name(&self) -> &'static str {
        match self {
            CallbackType::EmulationCycle => "OnEmulationCycle",
            CallbackType::Dot => "OnDot",
            CallbackType::Scanline => "OnScanline",
            CallbackType::Frame => "OnFrame",
            CallbackType::Instruction => "OnInstruction",
            CallbackType::Interrupt => "OnInterrupt",
            CallbackType::MemoryRead => "OnMemoryRead",
            CallbackType::MemoryWrite => "OnMemoryWrite",
            CallbackType::DmaStart => "OnDMAStart",
            CallbackType::DmaTransfer => "OnDMATransfer",
            CallbackType::DmaEnd => "OnDMAEnd",
            CallbackType::HdmaStart => "OnHDMAStart",
            CallbackType::HdmaTransfer => "OnHDMATransfer",
            CallbackType::HdmaEnd => "OnHDMAEnd",
        }
    }
    
    fn rust_fn_name(&self) -> &'static str {
        match self {
            CallbackType::EmulationCycle => "on_emulation_cycle",
            CallbackType::Dot => "on_dot",
            CallbackType::Scanline => "on_scanline",
            CallbackType::Frame => "on_frame",
            CallbackType::Instruction => "on_instruction",
            CallbackType::Interrupt => "on_interrupt",
            CallbackType::MemoryRead => "on_memory_read",
            CallbackType::MemoryWrite => "on_memory_write",
            CallbackType::DmaStart => "on_dma_start",
            CallbackType::DmaTransfer => "on_dma_transfer",
            CallbackType::DmaEnd => "on_dma_end",
            CallbackType::HdmaStart => "on_hdma_start",
            CallbackType::HdmaTransfer => "on_hdma_transfer",
            CallbackType::HdmaEnd => "on_hdma_end",
        }
    }
}

pub struct WatchpointEngine {
    pub initialized: bool,
    
    lua: Lua,
    callbacks: Callbacks,
}

impl WatchpointEngine {
    pub fn new() -> Self {
        let lua = Lua::new();
        
        Self {
            lua,
            initialized: false,
            callbacks: Callbacks {
                on_emulation_cycle: None,
                on_dot: None,
                on_scanline: None,
                on_frame: None,
                on_instruction: None,
                on_memory_read: None,
                on_memory_write: None,
                on_dma_start: None,
                on_dma_transfer: None,
                on_dma_end: None,
                on_hdma_start: None,
                on_hdma_transfer: None,
                on_hdma_end: None,
                on_interrupt: None,
            },
        }
    }
    
    pub fn init(&mut self, core: &mut Snemulator<Debugger>, control: &mut DebugControl) -> Result<()> {
        let globals = self.lua.globals();
        
        // Ignore emulator_api import, it is only for LSP aid
        globals.set("require", self.lua.create_function(|_, module: String| {
            if module.ends_with("snemulator_api") || module.ends_with("snemulator_api.lua") {
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError(format!("Module not found: {}", module)))
            }
        })?)?;
        
        // Register logging function
        globals.set("Log", self.lua.create_function(|_, msg: String| {
            log::debug!("{}", msg);
            Ok(())
        })?)?;
        
        let core_access = self.lua.create_userdata(SnemulatorInterface::new(core))?;
        globals.set("core", core_access)?;
        
        let control_access = self.lua.create_userdata(ControlInterface::new(control))?;
        globals.set("control", control_access)?;
        
        self.initialized = true;
        
        log::trace!("watchpoint engine initialized");
        
        Ok(())
    }
    
    pub fn unload_script(&mut self, core: &mut Snemulator<Debugger>, control: &mut DebugControl) {
        for callback_type in CallbackType::ALL {
            self.unregister_callback(*callback_type);
        }
        
        let _ = self.lua.globals().clear();
        self.init(core, control).ok();
    }
    
    pub fn load_script(&mut self, script: &str) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Engine must be initialized before loading script"));
        }
        
        let mut full_script = LUA_INCLUDE.to_string();
        full_script += script;
        
        self.lua.load(full_script).exec()?;

        let globals = self.lua.globals();
        
        let wp_func: Option<Function> = globals.get("OnLoad").ok();
        
        if let Some(func) = wp_func {
            match func.call::<()>(()) {
                Ok(_) => {},
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to run OnLoad: {}", e));
                }
            };
        }
        
        for callback_type in CallbackType::ALL {
            self.register_callback(*callback_type);
        }
        
        log::trace!("Loaded watchpoint script. Present callbacks:");
        
        for callback_type in CallbackType::ALL {
            log::trace!("  {}: {}", callback_type.lua_fn_name(), self.callbacks.get(*callback_type).is_some());
        }

        Ok(())
    }
    
    fn register_callback(&mut self, callback_type: CallbackType) {
        let callback = self.lua.globals().get(callback_type.lua_fn_name()).ok().flatten();
        
        self.callbacks.set(callback_type, callback);
    }
    
    fn unregister_callback(&mut self, callback_type: CallbackType) {
        self.callbacks.set(callback_type, None);
    }
    
    pub fn execute_callback<T: IntoLuaMulti>(&mut self, callback_type: CallbackType, args: T) {
        if let Some(func) = self.callbacks.get(callback_type) {
            match func.call::<()>(args) {
                Ok(_) => {},
                Err(e) => {
                    log::debug!("function '{}' failed to execute: {}", callback_type.lua_fn_name(), e);
                },
            }
        }
    }
    
    // pub fn on_emulation_cycle(&self) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_emulation_cycle, "OnEmulationCycle", ())
    // }
    
    // pub fn on_dot(&self) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_dot, "OnDot", ())
    // }
    
    // pub fn on_scanline(&self) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_scanline, "OnScanline", ())
    // }
    
    // pub fn on_frame(&self) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_frame, "OnFrame", ())
    // }
    
    // pub fn on_instruction(&self) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_instruction, "OnInstruction", ())
    // }
    
    // pub fn on_interrupt(&self, kind: CpuInterrupt) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_interrupt, "OnInterrupt", kind as u8)
    // }
    
    // pub fn on_memory_read(&self, addr: u32, value: u8) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_memory_read, "OnMemoryRead", (addr, value))
    // }
    
    // pub fn on_memory_write(&self, addr: u32, value: u8) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_memory_write, "OnMemoryWrite", (addr, value))
    // }
    
    // pub fn on_dma_start(&self, channel: usize) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_dma_start, "OnDMAStart", channel)
    // }
    
    // pub fn on_dma_transfer(&self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_dma_transfer, "OnDMATransfer", (channel, src_addr, dst_addr, value))
    // }
    
    // pub fn on_dma_end(&self, channel: usize) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_dma_end, "OnDMAEnd", channel)
    // }

    // pub fn on_hdma_start(&self, channel: usize) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_hdma_start, "OnHDMAStart", channel)
    // }
    
    // pub fn on_hdma_transfer(&self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_hdma_transfer, "OnHDMATransfer", (channel, src_addr, dst_addr, value))
    // }
    
    // pub fn on_hdma_end(&self, channel: usize) -> WatchpointAction {
    //     self.execute_fn(&self.callbacks.on_hdma_end, "OnHDMAEnd", channel)
    // }
}