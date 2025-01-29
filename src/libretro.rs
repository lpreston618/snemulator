
use libretro_rs::c_utf8::c_utf8;
use libretro_rs::retro::game::GameInfo;
use libretro_rs::retro::pixel::format::{ActiveFormat, XRGB8888};
use libretro_rs::retro::video::{ArrayFrameBuffer, FrameBuffer};
use libretro_rs::{ext, libretro_core};
use libretro_rs::retro::av::{GameGeometry, PixelFormat, SoftwareRenderEnabled, SystemAVInfo};
use libretro_rs::retro::env::GetAvInfo;
use libretro_rs::retro::{LoadGameExtraArgs, SystemInfo};
use libretro_rs::retro::log::{LogInterface, Logger, PlatformLogger};

use libretro_rs::retro::{
    self,
    Callbacks,
    InputsPolled,
};

// use libretro_rs::ffi::retro_log_level;

const SNES_VIDEO_WIDTH: u16 = 512;
const SNES_VIDEO_HEIGHT: u16 = 448;
const FRAME_BUFFER_LEN: usize = (SNES_VIDEO_WIDTH  as usize) * (SNES_VIDEO_HEIGHT as usize);

pub struct NemulatorCore {
    logger: PlatformLogger,

    frame_buffer: ArrayFrameBuffer<XRGB8888, FRAME_BUFFER_LEN, SNES_VIDEO_WIDTH>,
    rendering_mode: SoftwareRenderEnabled,
    pixel_format: ActiveFormat<XRGB8888>,
}


impl NemulatorCore {
    pub fn render_audio(&mut self, callbacks: &mut impl Callbacks) {
        // callbacks.upload_audio_frame(&audio_batch);
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

    // fn cycle(&mut self) {}

    fn cycle_frame(&mut self) {
        
    }
}


impl<'a> retro::Core<'a> for NemulatorCore {
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

    fn load_game<E: retro::env::LoadGame>(
        game: &GameInfo,
        args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
      ) -> Result<Self, retro::error::CoreError> {


        let LoadGameExtraArgs { 
            env, 
            pixel_format, 
            rendering_mode, 
            ..
        } = args;
        let pixel_format = env.set_pixel_format_xrgb8888(pixel_format)?;
        let mut logger = env.get_log_interface()?;
        let mut frame_buffer = ArrayFrameBuffer::new([XRGB8888::DEFAULT; FRAME_BUFFER_LEN]);

        for x in 0..SNES_VIDEO_WIDTH {
            let r = (((x as f32) / (SNES_VIDEO_WIDTH as f32)) * 255.0) as u8;

            for y in 0..SNES_VIDEO_HEIGHT {
                let g = (((y as f32) / (SNES_VIDEO_HEIGHT as f32)) * 255.0) as u8;
                let idx = (y as usize) * (SNES_VIDEO_WIDTH as usize) + x as usize;

                frame_buffer[idx].set_r(r);
                frame_buffer[idx].set_g(g);
            }
        }

        let mut core = Self {
            logger,
            frame_buffer,
            rendering_mode,
            pixel_format,
        };

        core.logger.info(c_utf8!("loaded game"));

        Ok(core)
        
        // todo!("Load Game");
    }

    fn get_system_av_info(&self, env: &mut impl GetAvInfo) -> SystemAVInfo {
        const WINDOW_SCALE: u16 = 4;
        const WINDOW_WIDTH: u16 = WINDOW_SCALE * SNES_VIDEO_WIDTH;
        const WINDOW_HEIGHT: u16 = WINDOW_SCALE * SNES_VIDEO_HEIGHT;
        SystemAVInfo::default_timings(GameGeometry::fixed(WINDOW_WIDTH, WINDOW_HEIGHT))
    }

    fn run(&mut self, env: &mut impl retro::env::Run, callbacks: &mut impl Callbacks) -> InputsPolled {
        let inputs_polled = self.update_input(callbacks);

        self.cycle_frame();

        // self.render_audio(callbacks);
        self.render_video(callbacks);
        
        inputs_polled
    }

    fn reset(&mut self, env: &mut impl retro::env::Reset) {
        todo!("Reset Core");
    }

    fn unload_game(self, env: &mut impl retro::env::UnloadGame) -> Self::Init {
        todo!("Unload Game");
    }

    fn load_without_content<E: retro::env::LoadGame>(
        args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
      ) -> Result<Self, retro::error::CoreError> {
        todo!("Load Without Content");
    }

    fn deinit(env: &mut impl retro::env::Deinit, init_state: Self::Init) {
        todo!("Deinit")
    }

    fn set_environment(env: &mut impl retro::env::SetEnvironment) {
        // todo!("Set Environment")
    }
}

libretro_core!(crate::libretro::NemulatorCore);