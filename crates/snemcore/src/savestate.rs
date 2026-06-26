use serde::{Deserialize, Serialize};

use crate::{scpu::Address, sppu::{AddressRemapping, BgMode, BgSettings, CMathOperator, Color, IncrSize, LayerSettings, M7FillMode, MasterSlave, ObjectSizeSelect, VideoType, VramIncMode, WindowColorRegion, WindowSettings}, ssmp::sdsp::{ADSRStage, GainMode}};

pub const MAGIC_SAVE_STATE_STRING: &[u8; 16] = b"SnemulatorSave:)"; 
pub const SAVE_STATE_VERSION: u32 = 0;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub magic_str: [u8; 16],
    pub version: u32,
    pub cpu: CpuState,
    pub ppu: PpuState,
    pub apu: ApuState,
    pub dma: DmaState,
    pub sram: Vec<u8>,
    pub wram: Vec<u8>,
    pub vram: Vec<u16>,
    pub aram: Vec<u8>,
    pub cgram: Vec<u8>,
    pub oam: Vec<u8>,
    pub cpu_open_bus: u8,
    pub apuio: [u8; 4],
    pub cpuio: [u8; 4],
    pub rom_hash: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CpuState {
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub sp: u16,
    pub pc: u16,
    pub pb: u8,
    pub db: u8,
    pub dp: u16,
    pub p: u8,
    pub e: bool,
    pub halted: bool,
    pub stopped: bool,
    pub waiting_for_interrupt: bool,
    pub clocks: usize,
}

#[derive(Serialize, Deserialize)]
pub struct PpuState {
    pub dot: usize,
    pub scanline: usize,
    pub frame: usize,
    pub clocks: usize,
    pub in_fblank: bool,
    pub screen_brightness: u8,
    pub obj_sprite_size: ObjectSizeSelect,
    pub name_base_addr: u16,
    pub name_secondary_base_addr: u16,
    pub oam_write_high_table: bool,
    pub internal_oam_addr: u16,
    pub priority_rotation: bool,
    pub priority_rotation_idx: u8,
    pub oam_data_latch: u8,
    pub bg3_mode1_priority: bool,
    pub bg_mode: BgMode,
    pub mosaic_size: u8,
    pub bg_settings: [BgSettings; 4],
    pub obj_settings: LayerSettings,
    pub col_window: WindowSettings,
    pub m7_latch: u8,
    pub bg_offset_latch: u8,
    pub bg_offset_x_latch: u8,
    pub m7_scroll_x: u16,
    pub m7_scroll_y: u16,
    pub vram_addr_inc_mode: VramIncMode,
    pub addr_remap_mode: AddressRemapping,
    pub addr_inc_size: IncrSize,
    pub vram_addr: u16,
    pub m7_tilemap_repeat: bool,
    pub m7_fill_mode: M7FillMode,
    pub m7_flip_bg_y: bool,
    pub m7_flip_bg_x: bool,
    pub m7_matrix_a: u16,
    pub mult_factor_16: u16,
    pub m7_matrix_b: u16,
    pub mult_factor_8: u8,
    pub m7_matrix_c: u16,
    pub m7_matrix_d: u16,
    pub m7_center_x: u16,
    pub m7_center_y: u16,
    pub cgram_toggle: bool,
    pub cgram_addr: u8,
    pub cgram_latch: u8,
    pub w1_left_pos: u8,
    pub w1_right_pos: u8,
    pub w2_left_pos: u8,
    pub w2_right_pos: u8,
    pub col_win_main_region: WindowColorRegion,
    pub col_win_sub_region: WindowColorRegion,
    pub sub_color_fixed: bool,
    pub use_direct_col: bool,
    pub cmath_operator: CMathOperator,
    pub cmath_half: bool,
    pub back_cmath_en: bool,
    pub fixed_color: Color,
    pub _external_sync: bool,
    pub ext_bg_en: bool,
    pub hi_res_en: bool,
    pub overscan_en: bool,
    pub obj_interlace_en: bool,
    pub screen_interlace_en: bool,
    pub multiply_result: u32,
    pub vram_latch: u16,
    pub h_counter_toggle: bool,
    pub h_counter_latch: u16,
    pub v_counter_toggle: bool,
    pub v_counter_latch: u16,
    pub sprite_overflow: bool,
    pub sprite_tile_overflow: bool,
    pub master_slave_state: MasterSlave,
    pub ppu1_version: u8,
    pub interlace_field: bool,
    pub counter_toggle: bool,
    pub video_type: VideoType,
    pub ppu2_version: u8,
}

#[derive(Serialize, Deserialize)]
pub struct ApuState {
    // Clocking info
    pub sample_cycle_accumulator: usize,
    pub spc_cycle_accumulator: usize,

    pub spc: SpcState,
    pub sdsp: SdspState,
    pub voices: [VoiceState; 8],
    pub timers: [TimerState; 3],
}

#[derive(Serialize, Deserialize)]
pub struct SpcState {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub dir_page: u16,
    pub stopped: bool,

    pub ipl_read_en: bool,
    pub sdsp_read_only: bool,
    pub sdsp_addr: u8,
}

#[derive(Serialize, Deserialize)]
pub struct SdspState {
    pub envelope_counter: usize,
    pub noise_output: u16,
    pub echo_ptr: usize,

    pub lmain_volume: u8,
    pub rmain_volume: u8,
    pub lecho_volume: u8,
    pub recho_volume: u8,
    pub key_on: u8,
    pub key_off: u8,
    pub soft_reset: bool,
    pub mute_all: bool,
    pub echo_en: bool,
    pub noise_freq: u8,
    pub echo_feedback: u8,
    pub unused: u8,
    pub sample_directory_page: u8,
    pub echo_page: u8,
    pub echo_delay_time: u8,
    pub fir_regs: [i8; 8],
}

#[derive(Serialize, Deserialize)]
pub struct VoiceState {
    pub lchannel_volume: u8,
    pub rchannel_volume: u8,
    pub pitch: u16,
    pub sample_source: u8,
    pub adsr_en: bool,
    pub adsr_decay: u8,
    pub adsr_attack: u8,
    pub adsr_sustain_level: u8,
    pub adsr_sustain_rate: u8,
    pub gain_reg_raw: u8,
    pub gain_fixed: u8,
    pub gain_rate: u8,
    pub gain_mode: GainMode,
    pub envelope: i16,
    pub sample_out_high: i16,
    pub ram_a: u8,
    pub ram_b: u8,
    pub end_of_sample_flag: bool,
    pub loop_flag: bool,
    pub pitchmod_en: bool,
    pub noise_en: bool,
    pub echo_en: bool,
    pub adsr_stage: ADSRStage,
    pub interpolation_idx: usize,
    pub brr_sample_buffer: [i16; 12],
    pub brr_group_addr: u16,
    pub brr_group_step: usize,
}

#[derive(Serialize, Deserialize)]
pub struct TimerState {
    pub enable: bool,
    pub target: u8,
    pub counter: u8,
    pub internal_counter: u8,
    pub clocks: usize,
}

#[derive(Serialize, Deserialize)]
pub struct DmaState {
    pub dma_en: bool,
    pub hdma_en: bool,
    pub hdma_pending: bool,
    pub hdma_needs_init: bool,
    pub dma_active_ch: usize,
    pub hdma_active_ch: usize,
    pub channels: [DmaChannelState; 8],
}

#[derive(Serialize, Deserialize)]
pub struct DmaChannelState {
    pub dma_en: bool,
    pub hdma_en: bool,
    pub reg_43n0_raw: u8,
    pub b_bus_addr: u8,
    pub a_bus_addr: Address,
    pub hdma_indirect_table_addr: Address,
    pub hdma_table_offset: u16,
    pub hdma_repeat_flag: bool,
    pub entry_scanline_count: u8,
    pub scanlines_left: u8,
    pub unused: u8,
    pub hdma_entry_just_loaded: bool,
    pub hdma_initialized: bool,
    pub hdma_do_transfer: bool,
    pub dma_bytes_transferred: usize,
}