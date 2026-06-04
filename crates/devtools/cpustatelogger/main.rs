use std::{io::Write, path::PathBuf};

use rfd::FileDialog;
use snemcore::{self, Snemulator, controller::{ControllerPlayer::Player1, JoypadButton}};

struct CpuStateLogger {
    output: bool,
    instr_count: usize,
    outstr: String,
}

impl CpuStateLogger {
    fn new() -> Self {
        Self {
            output: false,
            instr_count: 0,
            outstr: String::new(),
        }
    }

    fn write_to_file(self, path: &str) {
        let mut f = std::fs::File::create(path).unwrap();

        f.write(self.outstr.as_bytes());
    }

    fn state_str(&self, core: &mut snemcore::Snemulator<Self>) -> String {
        format!("{} {} {:X} {:X} {:X} {:X} {:X} {:X} {:X} {:X} {:X} {}",
            core.frame,
            self.instr_count,
            core.cpu.a,
            core.cpu.x,
            core.cpu.y,
            core.cpu.sp,
            core.cpu.pc,
            core.cpu.pb,
            core.cpu.db,
            core.cpu.dp,
            core.cpu.p,
            if core.cpu.e { 1 } else { 0 },
        )
    }
}

impl snemcore::probe::DebugProbe for CpuStateLogger {
    fn on_instruction(&mut self, core: &mut snemcore::Snemulator<Self>) {
        if self.output {
            let state = self.state_str(core) + "\n";
            self.outstr.push_str(&state);
        }
    }
}

fn main() {
    const START_FRAME: usize = 720;
    const END_FRAME: usize = 725;
    // const NUM_INSTRS: usize = 100000;

    let mut rom_path: Option<PathBuf> = None;
    
    for arg in std::env::args() {
        if arg.contains("-rom=") {
            rom_path = Some(arg["-rom=".len()..].into());
        }
    }
    
    let probe = CpuStateLogger::new();
    let mut core: Snemulator<CpuStateLogger> = Snemulator::with_probe(probe);
    
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
    
    core.load_rom(data).unwrap();
    core.power_on();

    for frame in 0..START_FRAME {
        if frame == 600 || frame == 660 {
            core.set_button(Player1, JoypadButton::A, true);
        }

        core.run_frame(None, None);

        if frame == 601 || frame == 661 {
            core.set_button(Player1, JoypadButton::A, false);
        }
    }

    core.probe.as_mut().unwrap().output = true;

    for _ in START_FRAME..END_FRAME {
        core.run_frame(None, None);
    }

    let probe = core.probe.take().unwrap();

    probe.write_to_file("cpustates.txt");

    println!("Done.");
}