use anyhow::Result;
use snemulator::app::EmulatorApp;

fn main() -> Result<()> {
    let mut app = EmulatorApp::new()?;
    
    app.run()
}
