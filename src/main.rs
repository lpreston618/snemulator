use anyhow::Result;
use log::info;
use crate::app::SnemulatorApp;

mod app;
mod core;
mod utils;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("Snemulator launched");
    
    let mut app = SnemulatorApp::new()?;
    
    let result = app.run();
    
    info!("App finished with result: {:?}", result);
    
    result
}
