use anyhow::Result;
use snemulator::app::SnemulatorApp;

fn main() -> Result<()> {
    let mut app = SnemulatorApp::new()?;
    
    app.run()
}
