use std::cell::RefCell;
use std::ffi::CStr;
use std::rc::Rc;

use crate::log::{LogLevel, SnemLogger};
use crate::system::cartridge::{self, Cartridge};
use crate::system::scpu;
use crate::system::ppu;
use crate::system::ssmp;

use libretro_rs::c_utf8::c_utf8;
use libretro_rs::retro::av::{
    GameGeometry, Message, SoftwareRenderEnabled, SystemAVInfo,
};
use libretro_rs::retro::env::GetAvInfo;
use libretro_rs::retro::error::CoreError;
use libretro_rs::retro::game::GameInfo;
use libretro_rs::retro::{LoadGameExtraArgs, SystemInfo};
use libretro_rs::{ext, libretro_core};

use libretro_rs::retro::{
    self,
    pixel::format::ActiveFormat,
    pixel::format::ORGB1555,
    Callbacks, InputsPolled,
};

use retro::framebuf::ResizableFrameBuffer;

const SNES_FRAME_WIDTH: usize = 512;
const SNES_FRAME_HEIGHT: usize = 448;
const FRAME_BUF_SIZE: usize = SNES_FRAME_WIDTH * SNES_FRAME_HEIGHT;
const AUDIO_FREQ: usize = 44100;
const AUDIO_BUFFER_SAMPLES: usize = AUDIO_FREQ / 60;

struct SnemulatorCore {
    logger: Rc<SnemLogger>,
    frame_buffer: ResizableFrameBuffer<ORGB1555, FRAME_BUF_SIZE>,
    pixel_format: ActiveFormat<ORGB1555>,
    rendering_mode: SoftwareRenderEnabled,
    audio_buffer: [i16; AUDIO_BUFFER_SAMPLES * 2],

    snem_cpu: scpu::Cpu65c816,
    snem_ppu: ppu::Ppu5C7x,
    snem_apu: ssmp::Spc700,

    frame_count: u64,

    #[cfg(feature = "warn-perf")]
    last_frame: std::time::Instant,
    #[cfg(feature = "warn-perf")]
    prev_fps: Vec<f32>,
}

fn screen_message(env: &mut impl retro::env::Run, message: &str, frames: u32) {
    let msg_str = format!("{}\0", message);
    let fps_count = unsafe { CStr::from_bytes_with_nul_unchecked(msg_str.as_bytes()) };
    let msg = Message::new(fps_count, frames);
    let _ = env.set_message(&msg);
}

impl SnemulatorCore {
    pub fn render_audio(&mut self, callbacks: &mut impl Callbacks) {
        callbacks.upload_audio_frame(&self.audio_buffer);
    }

    pub fn render_video(&mut self, callbacks: &mut impl Callbacks) {
        callbacks.upload_video_frame(&self.rendering_mode, &self.pixel_format, &self.frame_buffer);
    }

    pub fn update_input(&mut self, callbacks: &mut impl Callbacks) -> InputsPolled {
        let inputs_polled = callbacks.poll_inputs();

        // let p1_port = DevicePort::new(0);
        // let p2_port = DevicePort::new(1);

        // set_button!(self, callbacks, p1_controller, p1_port, A);
        // set_button!(self, callbacks, p1_controller, p1_port, B);
        // set_button!(self, callbacks, p1_controller, p1_port, Start);
        // set_button!(self, callbacks, p1_controller, p1_port, Select);
        // set_button!(self, callbacks, p1_controller, p1_port, Up);
        // set_button!(self, callbacks, p1_controller, p1_port, Down);
        // set_button!(self, callbacks, p1_controller, p1_port, Left);
        // set_button!(self, callbacks, p1_controller, p1_port, Right);

        // set_button!(self, callbacks, p2_controller, p2_port, A);
        // set_button!(self, callbacks, p2_controller, p2_port, B);
        // set_button!(self, callbacks, p2_controller, p2_port, Start);
        // set_button!(self, callbacks, p2_controller, p2_port, Select);
        // set_button!(self, callbacks, p2_controller, p2_port, Up);
        // set_button!(self, callbacks, p2_controller, p2_port, Down);
        // set_button!(self, callbacks, p2_controller, p2_port, Left);
        // set_button!(self, callbacks, p2_controller, p2_port, Right);

        inputs_polled
    }

    /// Clocks the PPU or the CPU a single time, depending on which one is
    /// supposed to be clocked next. Also clocks the APU as many times as 
    /// necessary for it to catch up.
    fn cycle(&mut self) {
        let ppu_clocks = self.snem_ppu.sys_clocks_left();
        let cpu_clocks = self.snem_cpu.sys_clocks_left();

        if ppu_clocks < cpu_clocks {
            self.snem_cpu.remove_clocks(ppu_clocks);
            self.snem_ppu.clock(&mut self.frame_buffer);
            self.snem_apu.clock(ppu_clocks);
        } else {
            self.snem_ppu.remove_clocks(cpu_clocks);
            self.snem_cpu.clock();
            self.snem_apu.clock(cpu_clocks);
        }
    }

    /// Clocks all components of the SNES until the PPU reports that the frame 
    /// is finished.
    fn cycle_frame(&mut self) {
        while !self.snem_ppu.frame_finished {
            self.cycle();
        }

        println!("Finished frame {}", self.frame_count);

        self.snem_ppu.frame_finished = false;
    }
}

impl<'a> retro::Core<'a> for SnemulatorCore {
    type Init = ();

    fn get_system_info() -> SystemInfo {
        SystemInfo::new(
            c_utf8!("Snemulator"),
            c_utf8!(env!("CARGO_PKG_VERSION")),
            ext!["sfc", "smc"],
        )
    }

    fn init(env: &mut impl retro::env::Init) -> Self::Init {}

    fn load_without_content<E: retro::env::LoadGame>(
        args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
    ) -> Result<Self, retro::error::CoreError> {
        let logger = Rc::new(
            SnemLogger::new(args.env.get_log_interface()?)
        );

        logger.log(LogLevel::Info, "loading Snemulator core with no content");

        args.env.set_hw_render_none()?;

        let mut frame_buffer = ResizableFrameBuffer::new();
        frame_buffer
            .resize(SNES_FRAME_WIDTH as u16 / 2, SNES_FRAME_HEIGHT as u16 / 2)
            .unwrap();
        let pixel_format = args.env.set_pixel_format_0rgb1555(args.pixel_format)?;
        let rendering_mode = args.rendering_mode;

        let ppu_data = Rc::new( ppu::PpuData::new(logger.clone()) );
        let apuio_regs = Rc::new( ssmp::ApuIORegs::new() );
        let snem_cpu = scpu::Cpu65c816::new(
            ppu_data.clone(),
            apuio_regs.clone(),
            logger.clone());
        let snem_ppu = ppu::Ppu5C7x::new(
            ppu_data.clone(),
            logger.clone());
        let snem_apu = ssmp::Spc700::new(
            apuio_regs.clone(), 
            logger.clone()
        );

        let core = SnemulatorCore {
            logger,
            frame_buffer,
            pixel_format,
            rendering_mode,
            audio_buffer: [0; AUDIO_BUFFER_SAMPLES * 2],

            snem_cpu,
            snem_ppu,
            snem_apu,

            frame_count: 0,

            #[cfg(feature = "warn-perf")]
            last_frame: std::time::Instant::now(),
            #[cfg(feature = "warn-perf")]
            prev_fps: Vec::new(),
        };

        Ok(core)
    }

    fn load_game<E: retro::env::LoadGame>(
        game: &GameInfo,
        args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
        ) -> Result<Self, retro::error::CoreError> {

        let mut core = SnemulatorCore::load_without_content(args)?;

        let path_str = if game.is_path() {
            game.as_path().unwrap().path().as_str()
        } else if game.is_data() {
            game.as_data().unwrap().path().unwrap().as_str()
        } else {
            core.logger.log(LogLevel::Error, "game provided is neither path nor data");
            return Err(CoreError::new());
        };

        core.logger.log(
            LogLevel::Info,
            format!("loading game from '{}'", path_str).as_str(),
        );

        let game_path = std::path::Path::new(&path_str);
        let cart_res = Cartridge::from_path(game_path);

        if let Err(msg) = cart_res {
            core.logger.log(LogLevel::Error, format!("failed to load game: {}", msg).as_str());
            return Err(CoreError::new());
        }

        let cart = cart_res.unwrap();

        core.logger.log(
            LogLevel::Info, 
            format!("loaded ROM with {:?} memory mapping", cart.mapping_mode()).as_str()
        );

        core.snem_cpu.load_cart(cart);
        core.snem_cpu.initialize();

        Ok(core)
    }

    fn get_system_av_info(&self, _env: &mut impl GetAvInfo) -> SystemAVInfo {
        SystemAVInfo::default_timings(GameGeometry::fixed(
            SNES_FRAME_WIDTH as u16,
            SNES_FRAME_HEIGHT as u16,
        ))
    }

    fn run(&mut self, _env: &mut impl retro::env::Run,
        callbacks: &mut impl Callbacks) -> InputsPolled {

        let inputs_polled = self.update_input(callbacks);

        self.cycle_frame();

        #[cfg(not(feature = "no-audio"))]
        self.render_audio(callbacks);
        #[cfg(not(feature = "no-video"))]
        self.render_video(callbacks);

        #[cfg(feature = "warn-perf")]
        {
            const MIN_AVG_FPS: f32 = 45.0;

            let fps = 1.0 / self.last_frame.elapsed().as_secs_f32();

            self.prev_fps.push(fps);
            
            if self.prev_fps.len() > 120 {
                let avg_fps = self.prev_fps.drain(0..60).sum::<f32>() / 60.0;

                if avg_fps < MIN_AVG_FPS {
                    self.logger.log(
                        LogLevel::Warn,
                        format!("Poor performance detected. Avg FPS: {:.04}", avg_fps).as_str()
                    );
                }
            }

            self.last_frame = std::time::Instant::now();
        }

        self.frame_count += 1;

        inputs_polled
    }

    fn reset(&mut self, _env: &mut impl retro::env::Reset) {
        self.logger.log(LogLevel::Info, "core reset");
        todo!("Reset Core");
    }

    fn unload_game(self, _env: &mut impl retro::env::UnloadGame) -> Self::Init {
        self.logger.log(LogLevel::Info, "unloading game");
    }

    fn set_environment(env: &mut impl retro::env::SetEnvironment) {
        let _ = env.set_support_no_game(true);
    }

    // fn deinit(env: &mut impl retro::env::Deinit, init_state: Self::Init) {}
}

// Look into implementing these for more functionality:
//      (found in retro/cores.rs)
// SaveStateCore
// DeviceTypeAwareCore
// CheatsCore
// GetMemoryRegionCore
// SpecialGameCore

libretro_core!(crate::libretro::SnemulatorCore);
