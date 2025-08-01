use std::rc::Rc;

use crate::audio;
use crate::controller::SnemController;
use crate::log::{LogLevel, SnemLogger};
use crate::system::cartridge::Cartridge;
use crate::system::scpu;
use crate::system::ppu;
use crate::system::ssmp;

use libretro_rs::c_utf8::c_utf8;
use libretro_rs::retro::av::{
    GameGeometry, SoftwareRenderEnabled, SystemAVInfo,
};
use libretro_rs::retro::device::JoypadButton;
use libretro_rs::retro::env::GetAvInfo;
use libretro_rs::retro::error::CoreError;
use libretro_rs::retro::game::GameInfo;
use libretro_rs::retro::{LoadGameExtraArgs, SystemInfo};
use libretro_rs::{ext, libretro_core};

use libretro_rs::retro::{
    self,
    pixel::format::ActiveFormat,
    pixel::format::RGB565,
    Callbacks, InputsPolled,
};

use retro::framebuf::ResizableFrameBuffer;

const SNES_FRAME_WIDTH: usize = 512;
const SNES_FRAME_HEIGHT: usize = 448;
const FRAME_BUF_SIZE: usize = SNES_FRAME_WIDTH * SNES_FRAME_HEIGHT;

struct SnemulatorCore {
    logger: Rc<SnemLogger>,
    frame_buffer: ResizableFrameBuffer<RGB565, FRAME_BUF_SIZE>,
    pixel_format: ActiveFormat<RGB565>,
    rendering_mode: SoftwareRenderEnabled,

    audio_buffer: Vec<i16>,
    audio_buffer_status: audio::AudioBufferStatus,

    snem_cpu: scpu::Cpu65c816,
    snem_ppu: ppu::Ppu5C7x,
    snem_smp: ssmp::Ssmp,

    p1_controller: SnemController,
    p2_controller: SnemController,

    last_frame: std::time::Instant,

    frame_count: u64,
}

impl SnemulatorCore {
    pub fn render_audio(&mut self, callbacks: &mut impl Callbacks) {
        callbacks.upload_audio_frame(&self.audio_buffer);

        self.audio_buffer.clear()
    }

    pub fn render_video(&mut self, callbacks: &mut impl Callbacks) {
        callbacks.upload_video_frame(&self.rendering_mode, &self.pixel_format, &self.frame_buffer);
    }

    pub fn update_input(&mut self, callbacks: &mut impl Callbacks) -> InputsPolled {
        macro_rules! set_button {
            ($controller:expr, $port:expr, $button:expr) => {
                $controller.update_button_state($button,
                    callbacks.is_joypad_button_pressed(
                        $port,
                        $button.into()
                    )
                )
            }
        }

        let inputs_polled = callbacks.poll_inputs();

        let p1_port = retro::device::DevicePort::new(0);
        let p2_port = retro::device::DevicePort::new(1);

        set_button!(self.p1_controller, p1_port, JoypadButton::A);
        set_button!(self.p1_controller, p1_port, JoypadButton::B);
        set_button!(self.p1_controller, p1_port, JoypadButton::X);
        set_button!(self.p1_controller, p1_port, JoypadButton::Y);
        set_button!(self.p1_controller, p1_port, JoypadButton::Up);
        set_button!(self.p1_controller, p1_port, JoypadButton::Down);
        set_button!(self.p1_controller, p1_port, JoypadButton::Left);
        set_button!(self.p1_controller, p1_port, JoypadButton::Right);
        set_button!(self.p1_controller, p1_port, JoypadButton::Select);
        set_button!(self.p1_controller, p1_port, JoypadButton::Start);
        set_button!(self.p1_controller, p1_port, JoypadButton::L1);
        set_button!(self.p1_controller, p1_port, JoypadButton::R1);

        set_button!(self.p2_controller, p2_port, JoypadButton::A);
        set_button!(self.p2_controller, p2_port, JoypadButton::B);
        set_button!(self.p2_controller, p2_port, JoypadButton::X);
        set_button!(self.p2_controller, p2_port, JoypadButton::Y);
        set_button!(self.p2_controller, p2_port, JoypadButton::Up);
        set_button!(self.p2_controller, p2_port, JoypadButton::Down);
        set_button!(self.p2_controller, p2_port, JoypadButton::Left);
        set_button!(self.p2_controller, p2_port, JoypadButton::Right);
        set_button!(self.p2_controller, p2_port, JoypadButton::Select);
        set_button!(self.p2_controller, p2_port, JoypadButton::Start);
        set_button!(self.p2_controller, p2_port, JoypadButton::L1);
        set_button!(self.p2_controller, p2_port, JoypadButton::R1);

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
        } else {
            self.snem_ppu.remove_clocks(cpu_clocks);
            self.snem_cpu.clock(self.frame_count as usize);
            if self.snem_cpu.poll_controllers {
                self.snem_cpu.latch_controller_states(
                    self.p1_controller.state_as_u16(),
                    self.p2_controller.state_as_u16()
                );

                self.snem_cpu.poll_controllers = false;
            }

            if self.snem_cpu.auto_read_controllers {
                self.snem_cpu.do_joypad_auto_read(
                    self.p1_controller.state_as_u16(),
                    self.p2_controller.state_as_u16()
                );

                self.snem_cpu.auto_read_controllers = false;
            }
        }

        if self.audio_buffer.len() < audio::MAX_AUDIO_BUFFER_SIZE
            || self.audio_buffer_status.underrun_likely {
            self.snem_smp.clock(&mut self.audio_buffer);
        }
    }

    /// Clocks all components of the SNES until the PPU reports that the frame 
    /// is finished.
    fn cycle_frame(&mut self) {
        while !self.snem_ppu.frame_finished {
            self.cycle();
        }

        // If we are likely to underrun the audio buffer even with the extra
        // clocking in self.cycle(), fill the audio buffer to a larger size just
        // to be safe. Doesn't eliminate all possibility of crackling, but
        // reduces it greatly.
        if self.audio_buffer_status.occupancy < audio::MIN_AUDIO_BUFFER_STATUS {
            let num_samples = audio::AUDIO_BUFFER_PANIC_FILL_SIZE
                .checked_sub(self.audio_buffer.len())
                .unwrap_or(0);

            self.snem_smp.generate_samples(
                &mut self.audio_buffer,
                num_samples,
            );
        }

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

    fn init(env: &mut impl retro::env::Init) -> Self::Init {
        unsafe {
            env.cmd::<u32, retro::audio::AudioBufferStatusCallback, retro::audio::AudioBufferStatusCallback>(
                libretro_rs::ffi::RETRO_ENVIRONMENT_SET_AUDIO_BUFFER_STATUS_CALLBACK,
                retro::audio::AudioBufferStatusCallback::new(
                    Some(crate::audio::audio_buffer_status_cb),
                )
            ).unwrap()
        };
    }

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
        let pixel_format = args.env.set_pixel_format_rgb565(args.pixel_format)?;
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
        let snem_smp = ssmp::Ssmp::new(
            apuio_regs.clone(), 
            logger.clone()
        );

        let core = SnemulatorCore {
            logger,
            frame_buffer,
            pixel_format,
            rendering_mode,

            audio_buffer: Vec::new(),
            audio_buffer_status: audio::get_audio_buffer_status(),

            snem_cpu,
            snem_ppu,
            snem_smp,

            p1_controller: SnemController::new(),
            p2_controller: SnemController::new(),

            last_frame: std::time::Instant::now(),

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

        self.audio_buffer_status = audio::get_audio_buffer_status();

        let inputs_polled = self.update_input(callbacks);

        self.cycle_frame();

        self.render_audio(callbacks);
        self.render_video(callbacks);

        // println!("fps: {}", 1.0 / self.last_frame.elapsed().as_secs_f32());

        self.frame_count += 1;
        self.last_frame = std::time::Instant::now();

        inputs_polled
    }

    fn reset(&mut self, _env: &mut impl retro::env::Reset) {
        self.logger.log(LogLevel::Info, "core reset");
        todo!("Reset Core");
    }

    fn unload_game(mut self, _env: &mut impl retro::env::UnloadGame) -> Self::Init {
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
