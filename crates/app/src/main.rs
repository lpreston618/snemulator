use anyhow::Result;
use clap::Parser;
use crate::app::SnemulatorApp;

mod app;
mod windows;

#[cfg(feature = "debug")]
mod debug;

#[derive(Parser)]
#[command(name = "snemulator", about = "SNES Emulator")]
pub struct SnemulatorArgs {
    #[arg(long)]
    pub rom: Option<String>,

    #[arg(long)]
    pub seed: Option<u64>,

    #[arg(long)]
    pub start_paused: bool,

    /// Enable debug mode
    #[cfg(feature = "debug")]
    #[arg(long)]
    pub debug: bool,
}

fn main() -> Result<()> {
    #[cfg(feature = "debug")]
    {
        let log_level = std::env::var("RUST_LOG").unwrap_or_default();
        match log_level.as_str() {
            "debug" | "trace" => {}
            _ => std::env::set_var("RUST_LOG", "debug"),
        }
    }

    env_logger::init();

    log::info!("Snemulator launched");

    let args = SnemulatorArgs::parse();

    let mut app = SnemulatorApp::new(args)?;

    let result = app.run();

    log::info!("App finished with result: {:?}", result);

    result
}