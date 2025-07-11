use std::ffi::{c_char, CStr};
use std::rc::Rc;
use std::thread::sleep;
use std::time;

use crate::system::ppu::{self, Ppu5C7x, PpuData};
use crate::log::{LogLevel, SnemLogger};

use libretro_rs::c_utf8::{c_utf8, CUtf8};
use libretro_rs::ffi::retro_log_level;
use libretro_rs::retro::error::CoreError;
use libretro_rs::retro::game::GameInfo;
use libretro_rs::retro::log::{LogInterface, Logger, PlatformLogger};
use libretro_rs::retro::video::FrameBuffer;
use libretro_rs::{ext, libretro_core};
use libretro_rs::retro::av::{GameGeometry, Message, PixelFormat, SoftwareRenderEnabled, SystemAVInfo};
use libretro_rs::retro::env::GetAvInfo;
use libretro_rs::retro::{LoadGameExtraArgs, SystemInfo};

use libretro_rs::retro::{
    self,
    Callbacks,
    InputsPolled,
    pixel::format::{ActiveFormat, XRGB8888},
    video::ArrayFrameBuffer,
};

use retro::framebuf::ResizableFrameBuffer;

use crate::system::scpu::Cpu65c816;

const SNES_FRAME_WIDTH: usize = 512;
const SNES_FRAME_HEIGHT: usize = 448;
const FRAME_BUF_SIZE: usize = SNES_FRAME_WIDTH*SNES_FRAME_HEIGHT;
const AUDIO_FREQ: usize = 44100;
const AUDIO_BUFFER_SAMPLES: usize = AUDIO_FREQ / 60;

struct SnemulatorCore {
    logger: SnemLogger,
    frame_buffer: ResizableFrameBuffer<
        XRGB8888, 
        FRAME_BUF_SIZE, 
    >,
    pixel_format: ActiveFormat<XRGB8888>,
    rendering_mode: SoftwareRenderEnabled,
    audio_buffer: [i16; AUDIO_BUFFER_SAMPLES*2],

    snem_cpu: Cpu65c816,
    snem_ppu: Ppu5C7x,

    last_frame: time::Instant,
    start: time::Instant,
    frame_count: u64,
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
        callbacks.upload_video_frame(
            &self.rendering_mode, 
            &self.pixel_format, 
            &self.frame_buffer,
        );
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

    fn cycle(&mut self) {
        self.snem_ppu.clock(&mut self.frame_buffer, &mut self.logger);
        self.snem_cpu.clock(&mut self.logger);
    }

    fn cycle_frame(&mut self) {
        // println!("Frame start");

        while !self.snem_ppu.frame_finished {
            self.cycle();
        }
    
        self.snem_cpu.flag_for_vblank_nmi();
        self.snem_ppu.frame_finished = false;

        // if self.frame_count == 4 {
        //     // ppu::dump_ppu_state(&self.snem_ppu);
        //     std::process::exit(1);
        // }

        // if self.frame_count == 100 {
        //     self.snem_ppu.dump_vram();
        // }
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

    fn init(env: &mut impl retro::env::Init) -> Self::Init {
        
    }

    fn load_without_content<E: retro::env::LoadGame>(
        args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
      ) -> Result<Self, retro::error::CoreError> {
        
        let mut logger = SnemLogger::new(args.env.get_log_interface()?);

        logger.log(LogLevel::Info, "Loading Snemulator core with no content.");

        args.env.set_hw_render_none()?;

        let mut frame_buffer = ResizableFrameBuffer::new();
        frame_buffer.resize(SNES_FRAME_WIDTH as u16 / 2, SNES_FRAME_HEIGHT as u16 / 2).unwrap();
        let pixel_format = args.env.set_pixel_format_xrgb8888(args.pixel_format)?;
        let rendering_mode = args.rendering_mode;

        let ppu_data = Rc::new(PpuData::new());
        let snem_cpu = Cpu65c816::new(ppu_data.clone());
        let snem_ppu = Ppu5C7x::new(ppu_data.clone());

        // for y in 0..SNES_FRAME_HEIGHT {
        //     for x in 0..SNES_FRAME_WIDTH {
        //         let idx = y*SNES_FRAME_WIDTH + x;
        //         let r = ((y as f64) / (SNES_FRAME_HEIGHT as f64) * 255.0) as u8;
        //         let g = ((x as f64) / (SNES_FRAME_WIDTH as f64) * 255.0) as u8;
        //         let col = XRGB8888::default()
        //             .with_r(r)
        //             .with_g(g)
        //             .with_b(0);
        //         frame_buffer[idx] = col;
        //     }
        // }

        let core = SnemulatorCore{
            logger,
            frame_buffer,
            pixel_format,
            rendering_mode,
            audio_buffer: [0; AUDIO_BUFFER_SAMPLES*2],

            snem_cpu,
            snem_ppu,

            last_frame: time::Instant::now(),
            start: time::Instant::now(),
            frame_count: 0,
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
            core.logger.log(LogLevel::Error, "Game provided is neither path nor data.");
            return Err(CoreError::new())
        };

        core.logger.log(LogLevel::Info, format!("Loading game from '{}'", path_str).as_str());

        // TODO: Load game here
        core.snem_cpu.temp_load_test();

        Ok(core)
    }

    fn get_system_av_info(&self, env: &mut impl GetAvInfo) -> SystemAVInfo {
        SystemAVInfo::default_timings(GameGeometry::fixed(SNES_FRAME_WIDTH as u16, SNES_FRAME_HEIGHT as u16))
    }

    fn run(&mut self, env: &mut impl retro::env::Run, callbacks: &mut impl Callbacks) -> InputsPolled {
        let inputs_polled = self.update_input(callbacks);

        self.cycle_frame();

        self.render_audio(callbacks);
        self.render_video(callbacks);

        println!("FPS: {}", 1.0 / self.last_frame.elapsed().as_secs_f32());

        self.last_frame = time::Instant::now();
        self.frame_count += 1;
        
        inputs_polled
    }

    fn reset(&mut self, env: &mut impl retro::env::Reset) {
        todo!("Reset Core");
    }

    fn unload_game(mut self, env: &mut impl retro::env::UnloadGame) -> Self::Init {
        self.logger.log(LogLevel::Info, "Unloading game..");
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