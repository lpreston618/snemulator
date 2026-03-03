use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::rc::Rc;

use crate::audio;
use crate::controller::SnemController;
use crate::log::{LogLevel, SnemLogger};
use crate::system::cartridge::Cartridge;
use crate::system::{scpu, sppu};
use crate::system::ssmp;

use libretro_rs::c_utf8::c_utf8;
use libretro_rs::retro::av::{
    GameGeometry, SoftwareRenderEnabled, SystemAVInfo, SystemTiming,
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

pub const IDEAL_FPS: usize = 60;

const SAVE_FILE_EXT: &str = ".srm";

static mut SAVE_DIRECTORY: Option<std::path::PathBuf> = None;

fn set_save_dir_path_str(path: std::path::PathBuf) { unsafe { SAVE_DIRECTORY = Some(path); } }
fn get_save_dir_path_str() -> Option<std::path::PathBuf> { unsafe { SAVE_DIRECTORY.clone() } }

struct SnemulatorCore {
    logger: Rc<SnemLogger>,
    frame_buffer: ResizableFrameBuffer<RGB565, FRAME_BUF_SIZE>,
    pixel_format: ActiveFormat<RGB565>,
    rendering_mode: SoftwareRenderEnabled,

    audio_buffer: Vec<i16>,
    // audio_buffer_status: audio::AudioBufferStatus,

    save_to_file: bool,
    save_data_path_str: Option<std::path::PathBuf>,

    snem_cpu: Option<scpu::Cpu65c816>,
    snem_ppu: Option<sppu::Ppu5C7x>,
    snem_smp: Option<ssmp::Ssmp>,

    p1_controller: SnemController,
    p2_controller: SnemController,

    last_frame: std::time::Instant,

    frame_count: u64,
}

impl SnemulatorCore {
    pub fn render_audio(&mut self, callbacks: &mut impl Callbacks) {
        callbacks.upload_audio_frame(self.audio_buffer.as_slice());
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

    // fn update_audio_buffer_status(&mut self) {
    //     let occupancy = self.audio_buffer_status.occupancy;
        
    //     self.audio_buffer_status = audio::get_audio_buffer_status();

    //     self.occupancy_delta = occupancy.checked_sub(self.audio_buffer_status.occupancy).unwrap_or(0);

    //     if self.audio_buffer_status.occupancy > self.max_occupancy {
    //         self.max_occupancy = self.audio_buffer_status.occupancy;

    //         self.occupancy_percent_samples = audio::FRONTEND_AUDIO_BUFFER_TARGET / self.max_occupancy;
    //     }
    // }

    /// Clocks the PPU or the CPU a single time, depending on which one is
    /// supposed to be clocked next. Clocks the S-SMP as well.
    fn cycle(&mut self, cpu: &mut scpu::Cpu65c816, ppu: &mut sppu::Ppu5C7x, smp: &mut ssmp::Ssmp) {
        let ppu_clocks = ppu.sys_clocks_left();
        let cpu_clocks = cpu.sys_clocks_left();
        let master_clocks = cpu_clocks.min(ppu_clocks);

        cpu.clock(master_clocks);
        ppu.clock(master_clocks, &mut self.frame_buffer);
        smp.clock(master_clocks, &mut self.audio_buffer);

        if cpu.poll_controllers {
            cpu.latch_controller_states(
                self.p1_controller.state_as_u16(),
                self.p2_controller.state_as_u16()
            );

            cpu.poll_controllers = false;
        }

        if cpu.auto_read_controllers {
            cpu.do_joypad_auto_read(
                self.p1_controller.state_as_u16(),
                self.p2_controller.state_as_u16()
            );

            cpu.auto_read_controllers = false;
        }
    }

    /// Clocks all components of the SNES until the PPU reports that the frame 
    /// is finished.
    fn cycle_frame(&mut self, cpu: &mut scpu::Cpu65c816, ppu: &mut sppu::Ppu5C7x, smp: &mut ssmp::Ssmp) {
        smp.start_frame();

        while !ppu.frame_finished {
            self.cycle(cpu, ppu, smp);
        }

        ppu.frame_finished = false;
    }

    fn try_load_sram(&mut self, game_path: &std::path::Path) {
        if let Some(mut dir_path) = get_save_dir_path_str() {
            let file_name = game_path.file_stem()
                .unwrap().to_str()
                .unwrap().to_owned();

            dir_path = dir_path.join(format!("{file_name}{SAVE_FILE_EXT}").as_str());

            self.save_data_path_str = Some(dir_path);

            let save_path = std::path::Path::new(self.save_data_path_str.as_ref().unwrap());

            let save_file = if save_path.exists() {
                Some(std::fs::File::open(save_path).unwrap())
            } else {
                None
            };

            if let Some(mut save_file) = save_file {
                let mut sram = Vec::new();

                let path_str = self.save_data_path_str.as_ref().unwrap().to_str().unwrap();

                match save_file.read_to_end(&mut sram) {
                    Ok(_) => {
                        self.snem_cpu.as_mut().unwrap().load_sram(sram);
                        self.logger.log(
                            LogLevel::Info, 
                            format!("loaded save file from '{path_str}'").as_str()
                        );
                    },
                    Err(_) => {
                        self.logger.log(
                            LogLevel::Error, 
                            format!("failed to load save file from '{path_str}'").as_str()
                        );
                    }
                }
            } else {
                self.logger.log(
                    LogLevel::Error, 
                    format!("failed to load save file from '{}'", save_path.to_str().unwrap()).as_str()
                );
            }
        } else {
            self.logger.log(
                LogLevel::Info, 
                "no save directory provided by frontend"
            );
        }
    }

    fn try_save_sram(&mut self) {
        let save_path = self.save_data_path_str.as_ref().unwrap();

        if let Ok(mut save_file) = std::fs::File::create(save_path) {
            let sram = self.snem_cpu.as_ref().unwrap().get_sram_as_slice();

            match save_file.write(sram) {
                Ok(_) => {
                    self.logger.log(
                        LogLevel::Info, 
                        format!("saved game to '{}'", save_path.to_str().unwrap()).as_str()
                    );
                },
                Err(_) => {
                    self.logger.log(
                        LogLevel::Error, 
                        format!("failed to load save file from '{}'", save_path.to_str().unwrap()).as_str()
                    );
                }
            }
        }
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
        if let Ok(Some(cstr)) = env.get_save_directory() {
            if let Ok(valid_str) = cstr.to_str() {
                set_save_dir_path_str(std::path::Path::new(valid_str).to_path_buf());
            }
        }
    }

    fn load_without_content<E: retro::env::LoadGame>(
        args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
    ) -> Result<Self, retro::error::CoreError> {

        let log_result = args.env.get_log_interface();
        let log_option = if let Ok(platform_logger) = log_result {
            Some(platform_logger)
        } else {
            None
        };

        let logger = Rc::new(SnemLogger::new(log_option));

        logger.log(LogLevel::Info, "loading Snemulator core with no content");

        let mut frame_buffer = ResizableFrameBuffer::new();
        frame_buffer
            .resize(SNES_FRAME_WIDTH as u16, SNES_FRAME_HEIGHT as u16)
            .unwrap();
        let rendering_mode = args.rendering_mode;
        let pixel_format = args.env.set_pixel_format_rgb565(args.pixel_format);

        if pixel_format.is_err() {
            logger.log(LogLevel::Error, "Failed to load core: could not set pixel format to rgb565");
            return Err(retro::error::CoreError::new());
        }

        let pixel_format = pixel_format.unwrap();

        let ppu_data = Rc::new( sppu::PpuData::new(logger.clone()) );
        let apuio_regs = Rc::new( ssmp::ApuIORegs::new() );
        let snem_cpu = scpu::Cpu65c816::new(
            ppu_data.clone(),
            apuio_regs.clone(),
            logger.clone());
        let snem_ppu = sppu::Ppu5C7x::new(
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
            // audio_buffer_status: audio::get_audio_buffer_status(),
            
            save_to_file: false,
            save_data_path_str: None,

            snem_cpu: Some(snem_cpu),
            snem_ppu: Some(snem_ppu),
            snem_smp: Some(snem_smp),

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

        core.save_to_file = cart.has_battery();

        core.logger.log(
            LogLevel::Info, 
            format!("loaded ROM with {:?} memory mapping", cart.mapping_mode()).as_str()
        );

        if core.save_to_file {
            core.try_load_sram(game_path);
        } else {
            core.logger.log(
                LogLevel::Info,
                "game has no save RAM on cart"
            )
        }

        core.snem_cpu.as_mut().unwrap().load_cart(cart);
        core.snem_cpu.as_mut().unwrap().initialize();

        Ok(core)
    }


    fn get_system_av_info(&self, _env: &mut impl GetAvInfo) -> SystemAVInfo {
        SystemAVInfo::new(
            GameGeometry::fixed(
                SNES_FRAME_WIDTH as u16,
                SNES_FRAME_HEIGHT as u16,
            ),
            SystemTiming::new(
                IDEAL_FPS as f64,
                audio::AUDIO_FREQ as f64,
            ),
        )
    }

    fn run(&mut self, _env: &mut impl retro::env::Run,
        callbacks: &mut impl Callbacks) -> InputsPolled {

        // self.update_audio_buffer_status();

        let inputs_polled = self.update_input(callbacks);

        if let (Some(mut cpu), Some(mut ppu), Some(mut smp))
            = (self.snem_cpu.take(), self.snem_ppu.take(), self.snem_smp.take()) {

            self.cycle_frame(&mut cpu, &mut ppu, &mut smp);

            self.snem_cpu = Some(cpu);
            self.snem_ppu = Some(ppu);
            self.snem_smp = Some(smp);
        }
        
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
        
        if self.save_to_file {
            self.try_save_sram();
            self.save_to_file = false;
            self.save_data_path_str = None;
        }
        
        self.snem_smp.as_mut().unwrap().finish();
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
