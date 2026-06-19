use anyhow::Result;
use clap::Parser;
use crate::app::SnemulatorApp;

mod app;
mod game;
mod menu;
mod theme;
mod settings;
mod app_utils;
mod ui_window;

#[cfg(feature = "debug")]
pub mod debug;

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

    #[arg(long)]
    pub theme: Option<String>,

    /// Start in debug mode
    #[cfg(feature = "debug")]
    #[arg(long)]
    pub debug: bool,
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