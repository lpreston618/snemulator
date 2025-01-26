use crate::*;

use libretro_rs::c_utf8::c_utf8;
use libretro_rs::retro::game::GameInfo;
use libretro_rs::{ext, libretro_core};
use libretro_rs::retro::av::SystemAVInfo;
use libretro_rs::retro::env::GetAvInfo;
use libretro_rs::retro::{LoadGameExtraArgs, SystemInfo};

use libretro_rs::retro::{
    self,
    Callbacks,
    device::{DevicePort, JoypadButton},
    InputsPolled,
    pixel::format::{ActiveFormat, XRGB8888},
    video::ArrayFrameBuffer,
};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::rc::Rc;


pub struct NemulatorCore {
    
}


impl NemulatorCore {
    pub fn render_audio(&mut self, callbacks: &mut impl Callbacks) {
        // callbacks.upload_audio_frame(&audio_batch);
    }

    pub fn render_video(&mut self, callbacks: &mut impl Callbacks) {
        // callbacks.upload_video_frame(
        //     &self.rendering_mode, 
        //     &self.pixel_format, 
        //     &self.frame_buffer,
        // );
    }

    pub fn update_input(&mut self, callbacks: &mut impl Callbacks) -> InputsPolled {
        todo!("Update Inputs");
        // let inputs_polled = callbacks.poll_inputs();

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
    
        // inputs_polled
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
        
        todo!("Load Game");
    }

    fn get_system_av_info(&self, env: &mut impl GetAvInfo) -> SystemAVInfo {
        // const WINDOW_SCALE: u16 = 8;
        // const WINDOW_WIDTH: u16 = WINDOW_SCALE * NES_SCREEN_WIDTH as u16;
        // const WINDOW_HEIGHT: u16 = WINDOW_SCALE * NES_SCREEN_HEIGHT as u16;
        // SystemAVInfo::default_timings(GameGeometry::fixed(WINDOW_WIDTH, WINDOW_HEIGHT))
        todo!("Get Sys AV Info");
    }

    fn run(&mut self, env: &mut impl retro::env::Run, callbacks: &mut impl Callbacks) -> InputsPolled {
        todo!("Run Core");
        // let inputs_polled = self.update_input(callbacks);

        // self.cycle_frame();

        // self.render_audio(callbacks);
        // self.render_video(callbacks);
        
        // inputs_polled
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
        todo!("Set Environment")
    }
}

libretro_core!(crate::libretro::NemulatorCore);