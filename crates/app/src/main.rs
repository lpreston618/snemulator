use anyhow::Result;
use log::info;
use crate::app::SnemulatorApp;

mod app;
mod windows;

#[cfg(feature = "debug")]
mod debug;

fn main() -> Result<()> {
    #[cfg(feature = "debug")]
    {   
        let log_level = std::env::var("RUST_LOG").unwrap_or_default();
        
        match log_level.as_str() {
            "debug" | "trace" => {},
            _ => std::env::set_var("RUST_LOG", "debug"),
        }
    }
    
    env_logger::init();
    
    info!("Snemulator launched");
    
    let mut app = SnemulatorApp::new()?;
    
    let result = app.run();
    
    info!("App finished with result: {:?}", result);
    
    result
}