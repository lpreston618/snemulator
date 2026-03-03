use anyhow::Result;
use log::info;
use snemulator::app::SnemulatorApp;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("Snemulator launched");
    
    let mut app = SnemulatorApp::new()?;
    
    let result = app.run();
    
    info!("App finished with result: {:?}", result);
    
    result
}
