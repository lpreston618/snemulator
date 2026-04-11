use std::{fmt::Write, path::PathBuf};

use rfd::FileDialog;
use snemcore::Snemulator;

fn main() {
    let mut iters = 1;
    let mut emulated_seconds = 60;
    let mut rom_path: Option<PathBuf> = None;
    
    for arg in std::env::args() {
        if arg.contains("-iters=") {
            iters = arg["-iters=".len()..].parse().ok().unwrap_or(iters);
        }
        
        if arg.contains("-seconds=") {
            emulated_seconds = arg["-seconds=".len()..].parse().ok().unwrap_or(emulated_seconds);
        }
        
        if arg.contains("-rom=") {
            rom_path = Some(arg["-rom=".len()..].into());
        }
    }
    
    let mut snem_core = Snemulator::new();
    
    let data = if let Some(rom_path) = rom_path {
        std::fs::read(&rom_path).unwrap()
    } else {
        std::fs::read(FileDialog::new()
            .add_filter("ROM", &["sfc", "smc"])
            .set_directory("/")
            .pick_file()
            .unwrap())
        .unwrap()
    };
    
    snem_core.load_rom(data).unwrap();
    snem_core.power_on();
    
    let mut speedups: Vec<f32> = Vec::new();
    
    for _ in 0..iters {        
        let speedup = speed_test(emulated_seconds, &mut snem_core);
        
        speedups.push(speedup);
    }
    
    println!("Avg Speedup: {:.4}x", speedups.iter().sum::<f32>() / (speedups.len() as f32));
}

pub fn speed_test(emulated_seconds: u64, snem_core: &mut snemcore::Snemulator) -> f32 {
    let mut frame_buffer = vec![0u8; 512 * 448 * 4];
    let mut audio_buffer = Vec::new();
    
    let pb = indicatif::ProgressBar::new(emulated_seconds);
    pb.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})\n{msg}")
        .unwrap()
        .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    
    for _ in 0..emulated_seconds {
        for _ in 0..60 {
            snem_core.run_frame(Some(&mut frame_buffer), Some(&mut audio_buffer));
        }
        
        pb.inc(1);
    }
    
    let elapsed = pb.elapsed().as_secs_f32();
    let emulation_speedup = (emulated_seconds as f32) / elapsed;
    pb.finish_with_message(format!("Emulated {} frames in {:.2} seconds ({:.4}x real time speed)", emulated_seconds, elapsed, emulation_speedup));

    emulation_speedup
}