use std::marker::PhantomData;

use mlua::{AnyUserData, Function, IntoLuaMulti, Lua, MultiValue, Result, UserData, Value::Nil};
use snemcore::{Snemulator, probe::DebugProbe};

use crate::debug::watchpoints::interface::SnemulatorInterface;

#[derive(Clone, Copy)]
pub enum WatchpointAction {
    Continue,
    Break,
}

impl UserData for WatchpointAction {}

pub struct WatchpointEngine<P: DebugProbe> {
    lua: Lua,
    script_loaded: bool,
    
    _phantom_probe: PhantomData<P>,
}

impl<P: DebugProbe> WatchpointEngine<P> {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        
        register_emulator_api(&lua)?;
        
        Ok(Self { lua, script_loaded: false, _phantom_probe: PhantomData {} })
    }
    
    pub fn load_script(&mut self, script: &str) -> Result<()> {
        // Load and compile the script
        let res = self.lua.load(script).exec();

        self.script_loaded = res.is_ok();

        res
    }
    
    fn execute_wp_fn<T: IntoLuaMulti>(&self, core: &Snemulator<P>, fn_name: &str, args: T) -> WatchpointAction {
        // Set up emulator access
        let globals = self.lua.globals();
        
        let wp_func: Option<Function> = globals.get(fn_name).ok();
        
        if wp_func.is_none() {
            return WatchpointAction::Continue;
        }
        
        let wp_func = wp_func.unwrap();
        
        let action = self.lua.scope(|scope| {
            let core_access = scope.create_userdata(SnemulatorInterface::new(core))?;

            globals.set("core", core_access)?;
            
            let result: AnyUserData = wp_func.call(args)?;
            let action_value = result.borrow::<WatchpointAction>()?;
            let action = *action_value;
            
            globals.set("core", Nil)?;
            
            Ok(action)
        });
        
        match action {
            Ok(action) => action,
            Err(e) => {
                log::debug!("function '{}' failed to execute: {}", fn_name, e);
                WatchpointAction::Break
            },
        }
    }
    
    // pub fn on_init(&self, core: &Emulator) -> Result<WatchpointAction> {
    //     self.execute_wp_fn(core, "OnInit", ())
    // }
    
    pub fn on_emulation_cycle(&self, core: &Snemulator<P>) -> WatchpointAction {
        self.execute_wp_fn(core, "OnEmulationCycle", ())
    }
    
    pub fn on_memory_write(&self, core: &Snemulator<P>, addr: u32, value: u8) -> WatchpointAction {
        self.execute_wp_fn(core, "OnMemoryWrite", (addr, value))
    }
    
    pub fn on_frame(&self, core: &Snemulator<P>) -> WatchpointAction {
        self.execute_wp_fn(core, "OnFrame", ())
    }
}

fn register_emulator_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    
    // Ignore emulator_api import, it is only for LSP aid
    globals.set("require", lua.create_function(|_, module: String| {
        if module.ends_with("snemulator_api") || module.ends_with("snemulator_api.lua") {
            Ok(())
        } else {
            Err(mlua::Error::RuntimeError(format!("Module not found: {}", module)))
        }
    })?)?;
    
    // Register logging function
    globals.set("Log", lua.create_function(|_, msg: String| {
        log::debug!("{}", msg);
        Ok(())
    })?)?;
    
    let action_table = lua.create_table()?;
    action_table.set("Continue", lua.create_userdata(WatchpointAction::Continue)?)?;
    action_table.set("Break", lua.create_userdata(WatchpointAction::Break)?)?;
    globals.set("ACTION", action_table)?;
    
    // Disable print to stdout
    lua.globals().set("print", lua.create_function(|_, _: MultiValue| Ok(()))?)?;
    
    Ok(())
}