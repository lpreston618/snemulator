use anyhow::Result;
use clap::Parser;
use crate::app::SnemulatorApp;

mod app;
mod windows;

#[derive(Parser)]
#[command(name = "snemulator", about = "SNES Emulator")]
pub struct SnemulatorArgs {
    #[arg(long)]
    pub rom: Option<String>,

    #[arg(long)]
    pub seed: Option<u64>,

    #[arg(long)]
    pub start_paused: bool,

    #[arg(long)]
    pub no_audio: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    log::info!("Snemulator launched");

    let args = SnemulatorArgs::parse();

    let mut app = SnemulatorApp::new(args)?;

    let result = app.run();

    log::info!("App finished with result: {:?}", result);

    result
}