use serde::{Deserialize, Serialize};

use crate::{clr_bit_n, get_bit_n, savestate, set_bit_n, set_byte_n};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum HVTimerIRQ {
    #[default]
    None, // Ignore H/V Timers
    HTimer, // IRQ when H counter == HTIME
    VTimer, // IRQ when V counter == VTIME and H counter == 0
    Both,   // IRQ when V counter == VTIME and H counter == HTIME
}

#[derive(Default)]
pub struct CpuIoRegs {
    // $2181 - LLLL LLLL
    // $2182 - MMMM MMMM
    // $2183 - .... ...H
    pub wram_address: usize,

    // $4016
    pub latch_controllers: bool,

    // $4200
    pub vblank_nmi_en: bool,
    pub hv_timer_irq_mode: HVTimerIRQ,
    pub joypad_autoread_en: bool,

    // $4202
    pub mult_factor1: u8,

    // $4203
    pub mult_factor2: u8,

    // $4204-$4205
    pub div_numer: u16,

    // $4206
    pub div_denom: u8,

    // $4207-$4208
    pub h_counter_target: u16,
    // $4209-$420A
    pub v_counter_target: u16,

    // $420D
    pub fast_rom_en: bool,

    // $4210
    pub vblank_nmi_flag: bool,

    // $4211
    pub hv_timer_irq_flag: bool,

    // $4212
    pub vblank_flag: bool,
    pub hblank_flag: bool,
    pub joypad_autoread_flag: bool,

    // $4201 (Write) and $4213 (Read)
    pub raw_rdwrio: u8,
    pub joy1_io: bool,
    pub joy2_io: bool,

    // $4214-$4215
    pub div_quotient: u16,

    // $4216-$4217
    pub mult_result: u16, // Also the division remainder

    pub nmi_pending: bool,
}

impl CpuIoRegs {
    pub fn power_on(&mut self) {
        self.write_4200(0);
        self.write_4201(0xFF);
        self.write_4202(0xFF);
        self.write_4203(0xFF);
        self.write_4204(0xFF);
        self.write_4205(0xFF);
        self.write_4206(0xFF);
        self.write_4207(0xFF);
        self.write_4208(1);
        self.write_4209(0xFF);
        self.write_420A(1);
        self.write_420D(0);

        self.vblank_nmi_flag = false;
        self.vblank_flag = false;
        self.hblank_flag = true;
        self.hv_timer_irq_flag = false;
        self.nmi_pending = false;
        self.div_quotient = 0;
        self.mult_result = 0;
    }

    pub fn reset(&mut self) {
        self.write_4200(0);
        self.write_4201(0xFF);

        self.vblank_nmi_flag = false;
        self.vblank_flag = false;
        self.hblank_flag = true;
        self.hv_timer_irq_flag = false;
        self.nmi_pending = false;
    }

    pub fn save_state(&self) -> savestate::CpuIoState {
        savestate::CpuIoState {
            wram_address: self.wram_address,
            latch_controllers: self.latch_controllers,
            vblank_nmi_en: self.vblank_nmi_en,
            hv_timer_irq_mode: self.hv_timer_irq_mode,
            joypad_autoread_en: self.joypad_autoread_en,
            mult_factor1: self.mult_factor1,
            mult_factor2: self.mult_factor2,
            div_numer: self.div_numer,
            div_denom: self.div_denom,
            h_counter_target: self.h_counter_target,
            v_counter_target: self.v_counter_target,
            fast_rom_en: self.fast_rom_en,
            vblank_nmi_flag: self.vblank_nmi_flag,
            hv_timer_irq_flag: self.hv_timer_irq_flag,
            vblank_flag: self.vblank_flag,
            hblank_flag: self.hblank_flag,
            joypad_autoread_flag: self.joypad_autoread_flag,
            raw_rdwrio: self.raw_rdwrio,
            joy1_io: self.joy1_io,
            joy2_io: self.joy2_io,
            div_quotient: self.div_quotient,
            mult_result: self.mult_result,
            nmi_pending: self.nmi_pending,
        }
    }

    pub fn load_state(&mut self, state: &savestate::CpuIoState, _version: u32) {
        self.wram_address = state.wram_address;
        self.latch_controllers = state.latch_controllers;
        self.vblank_nmi_en = state.vblank_nmi_en;
        self.hv_timer_irq_mode = state.hv_timer_irq_mode;
        self.joypad_autoread_en = state.joypad_autoread_en;
        self.mult_factor1 = state.mult_factor1;
        self.mult_factor2 = state.mult_factor2;
        self.div_numer = state.div_numer;
        self.div_denom = state.div_denom;
        self.h_counter_target = state.h_counter_target;
        self.v_counter_target = state.v_counter_target;
        self.fast_rom_en = state.fast_rom_en;
        self.vblank_nmi_flag = state.vblank_nmi_flag;
        self.hv_timer_irq_flag = state.hv_timer_irq_flag;
        self.vblank_flag = state.vblank_flag;
        self.hblank_flag = state.hblank_flag;
        self.joypad_autoread_flag = state.joypad_autoread_flag;
        self.raw_rdwrio = state.raw_rdwrio;
        self.joy1_io = state.joy1_io;
        self.joy2_io = state.joy2_io;
        self.div_quotient = state.div_quotient;
        self.mult_result = state.mult_result;
        self.nmi_pending = state.nmi_pending;
    }

    pub fn write_2181(&mut self, value: u8) {
        set_byte_n!(self.wram_address, value as usize, 0);
    }

    pub fn write_2182(&mut self, value: u8) {
        set_byte_n!(self.wram_address, value as usize, 1);
    }

    pub fn write_2183(&mut self, value: u8) {
        if get_bit_n!(value, 0) {
            set_bit_n!(self.wram_address, 16);
        } else {
            clr_bit_n!(self.wram_address, 16);
        }
    }

    pub fn write_4200(&mut self, value: u8) {
        let old_nmi_en = self.vblank_nmi_en;

        self.vblank_nmi_en = get_bit_n!(value, 7);
        self.joypad_autoread_en = get_bit_n!(value, 0);

        self.hv_timer_irq_mode = match (value >> 4) & 3 {
            0 => HVTimerIRQ::None,
            1 => HVTimerIRQ::HTimer,
            2 => HVTimerIRQ::VTimer,
            3 => HVTimerIRQ::Both,
            _ => unreachable!(),
        };

        if self.vblank_nmi_en && !old_nmi_en && self.vblank_nmi_flag {
            self.nmi_pending = true;
        }
    }

    pub fn write_4201(&mut self, value: u8) {
        self.raw_rdwrio = value;
        self.joy2_io = get_bit_n!(value, 7);
        self.joy1_io = get_bit_n!(value, 6);
    }

    pub fn write_4202(&mut self, value: u8) {
        self.mult_factor1 = value;
    }

    pub fn write_4203(&mut self, value: u8) {
        self.mult_factor2 = value;
        // TODO: Make mult circuit take cycles to compute result
        self.mult_result = self.mult_factor1 as u16 * self.mult_factor2 as u16;
    }

    pub fn write_4204(&mut self, value: u8) {
        set_byte_n!(self.div_numer, value as u16, 0);
    }
    pub fn write_4205(&mut self, value: u8) {
        set_byte_n!(self.div_numer, value as u16, 1);
    }

    pub fn write_4206(&mut self, value: u8) {
        self.div_denom = value;
        if self.div_denom == 0 {
            self.div_quotient = 0xFFFF;
            self.mult_result = self.div_numer;
        } else {
            self.div_quotient = self.div_numer / (self.div_denom as u16);
            self.mult_result = self.div_numer % (self.div_denom as u16);
        }
    }

    pub fn write_4207(&mut self, value: u8) {
        set_byte_n!(self.h_counter_target, value as u16, 0);
    }

    pub fn write_4208(&mut self, value: u8) {
        set_byte_n!(self.h_counter_target, (value & 1) as u16, 1);
    }

    pub fn write_4209(&mut self, value: u8) {
        set_byte_n!(self.v_counter_target, value as u16, 0);
    }

    #[allow(non_snake_case)]
    pub fn write_420A(&mut self, value: u8) {
        set_byte_n!(self.v_counter_target, (value & 1) as u16, 1);
    }

    #[allow(non_snake_case)]
    pub fn write_420D(&mut self, value: u8) {
        self.fast_rom_en = get_bit_n!(value, 0);
    }
}
