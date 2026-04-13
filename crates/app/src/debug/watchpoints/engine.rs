use anyhow::Result;

use std::marker::PhantomData;

use crate::debug::watchpoints::luainclude::LUA_INCLUDE;
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

pub struct WatchpointEngine<P: DebugProbe> {
    pub initialized: bool,
    
    lua: Lua,
    callbacks: Callbacks,
    
    _phantom_probe: PhantomData<P>,
}

impl<P: DebugProbe + 'static> WatchpointEngine<P> {
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
            _phantom_probe: PhantomData {},
        }
    }
    
    pub fn init(&mut self, core: &mut Snemulator<P>) -> Result<()> {
        let globals = self.lua.globals();
        
        // Ignore emulator_api import, it is only for LSP aid
        globals.set("require", self.lua.create_function(|_, module: String| {
            if module.ends_with("snemulator_api") || module.ends_with("snemulator_api.lua") {
                Ok(())
            } else {
                Err(mlua::Error::RuntimeError(format!("Module not found: {}", module)))
            }
        })
        .map_err(|e| anyhow::anyhow!("{}", e))?)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // Register logging function
        globals.set("Log", self.lua.create_function(|_, msg: String| {
            log::debug!("{}", msg);
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{}", e))?)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        let action_table = self.lua.create_table()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        action_table.set("Continue", self.lua.create_userdata(WatchpointAction::Continue)
            .map_err(|e| anyhow::anyhow!("{}", e))?)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        action_table.set("Break", self.lua.create_userdata(WatchpointAction::Break)
            .map_err(|e| anyhow::anyhow!("{}", e))?)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        globals.set("ACTION", action_table)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        let core_access = self.lua.create_userdata(SnemulatorInterface::new(core))
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    
        globals.set("core", core_access)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        self.initialized = true;
        
        log::trace!("watchpoint engine initialized");
        
        Ok(())
    }
    
    pub fn unload_script(&mut self, core: &mut Snemulator<P>) {
        let _ = self.lua.globals().clear();
        self.init(core).ok();
    }
    
    pub fn load_script(&mut self, script: &str) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Engine must be initialized before loading script"));
        }
        
        let mut full_script = LUA_INCLUDE.to_string();
        full_script += script;
        
        let res = self.lua.load(full_script).exec()
            .map_err(|e| anyhow::anyhow!("{}", e));

        if res.is_ok() {        
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
        }
        
        let globals = self.lua.globals();
        
        self.callbacks.on_emulation_cycle = globals.get("OnEmulationCycle").ok();
        self.callbacks.on_dot             = globals.get("OnDot").ok();
        self.callbacks.on_scanline        = globals.get("OnScanline").ok();
        self.callbacks.on_frame           = globals.get("OnFrame").ok();
        self.callbacks.on_instruction     = globals.get("OnInstruction").ok();
        self.callbacks.on_interrupt       = globals.get("OnInterrupt").ok();
        self.callbacks.on_memory_read     = globals.get("OnMemoryRead").ok();
        self.callbacks.on_memory_write    = globals.get("OnMemoryWrite").ok();
        self.callbacks.on_dma_start       = globals.get("OnDMAStart").ok();
        self.callbacks.on_dma_transfer    = globals.get("OnDMATransfer").ok();
        self.callbacks.on_dma_end         = globals.get("OnDMAEnd").ok();
        self.callbacks.on_hdma_start      = globals.get("OnHDMAStart").ok();
        self.callbacks.on_hdma_transfer   = globals.get("OnHDMATransfer").ok();
        self.callbacks.on_hdma_end        = globals.get("OnHDMAEnd").ok();
        
        log::trace!("Loaded watchpoint script. Present callbacks:");
        log::trace!("  on_emulation_cycle: {}", self.callbacks.on_emulation_cycle.is_some());
        log::trace!("  on_dot: {}",             self.callbacks.on_dot.is_some());
        log::trace!("  on_scanline: {}",        self.callbacks.on_scanline.is_some());
        log::trace!("  on_frame_end: {}",       self.callbacks.on_frame.is_some());
        log::trace!("  on_instruction: {}",     self.callbacks.on_instruction.is_some());
        log::trace!("  on_interrupt: {}",       self.callbacks.on_interrupt.is_some());
        log::trace!("  on_memory_read: {}",     self.callbacks.on_memory_read.is_some());
        log::trace!("  on_memory_write: {}",    self.callbacks.on_memory_write.is_some());
        log::trace!("  on_dma_start: {}",       self.callbacks.on_dma_start.is_some());
        log::trace!("  on_dma_transfer: {}",    self.callbacks.on_dma_transfer.is_some());
        log::trace!("  on_dma_end: {}",         self.callbacks.on_dma_end.is_some());
        log::trace!("  on_hdma_start: {}",      self.callbacks.on_hdma_start.is_some());
        log::trace!("  on_hdma_transfer: {}",   self.callbacks.on_hdma_transfer.is_some());
        log::trace!("  on_hdma_end: {}",        self.callbacks.on_hdma_end.is_some());

        res
    }
    
    fn try_execute_fn<T: IntoLuaMulti>(&self, wp_func: &Function, args: T) -> Result<WatchpointAction> {        
        let result: AnyUserData = match wp_func.call(args) {
            Ok(data) => data,
            Err(e) => {
                return Err(anyhow::anyhow!("Expected return type Action, {}", e));
            }
        };
        let action_value = result.borrow::<WatchpointAction>();
        let action = match action_value {
            Ok(a) => *a,
            Err(_) => WatchpointAction::Continue, // Ignore other return types
        };
        
        Ok(action)
    }
    
    fn execute_fn<T: IntoLuaMulti>(&self, func: &Option<Function>, name: &str, args: T) -> WatchpointAction {
        if let Some(func) = func {
            let action = self.try_execute_fn(func, args);
            
            match action {
                Ok(action) => action,
                Err(e) => {
                    log::debug!("function '{}' failed to execute: {}", name, e);
                    WatchpointAction::Break
                },
            }
        } else {
            WatchpointAction::Continue
        }
    }
    
    pub fn on_emulation_cycle(&self) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_emulation_cycle, "OnEmulationCycle", ())
    }
    
    pub fn on_dot(&self) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_dot, "OnDot", ())
    }
    
    pub fn on_scanline(&self) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_scanline, "OnScanline", ())
    }
    
    pub fn on_frame(&self) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_frame, "OnFrame", ())
    }
    
    pub fn on_instruction(&self) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_instruction, "OnInstruction", ())
    }
    
    pub fn on_interrupt(&self, kind: CpuInterrupt) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_interrupt, "OnInterrupt", kind as u8)
    }
    
    pub fn on_memory_read(&self, addr: u32, value: u8) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_memory_read, "OnMemoryRead", (addr, value))
    }
    
    pub fn on_memory_write(&self, addr: u32, value: u8) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_memory_write, "OnMemoryWrite", (addr, value))
    }
    
    pub fn on_dma_start(&self, channel: usize) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_dma_start, "OnDMAStart", channel)
    }
    
    pub fn on_dma_transfer(&self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_dma_transfer, "OnDMATransfer", (channel, src_addr, dst_addr, value))
    }
    
    pub fn on_dma_end(&self, channel: usize) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_dma_end, "OnDMAEnd", channel)
    }

    pub fn on_hdma_start(&self, channel: usize) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_hdma_start, "OnHDMAStart", channel)
    }
    
    pub fn on_hdma_transfer(&self, channel: usize, src_addr: u32, dst_addr: u32, value: u8) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_hdma_transfer, "OnHDMATransfer", (channel, src_addr, dst_addr, value))
    }
    
    pub fn on_hdma_end(&self, channel: usize) -> WatchpointAction {
        self.execute_fn(&self.callbacks.on_hdma_end, "OnHDMAEnd", channel)
    }
}