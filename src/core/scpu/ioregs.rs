#[derive(Clone, Copy, Debug, Default)]
pub enum HVTimerIRQ {
    #[default]
    None,   // Ignore H/V Timers
    HTimer, // IRQ when H counter == HTIME
    VTimer, // IRQ when V counter == VTIME and H counter == 0
    Both,   // IRQ when V counter == VTIME and H counter == HTIME
}

#[derive(Debug, Default)]
pub struct CpuIoRegs {    
    // $4016
    pub latch_controllers: bool,
    
    // $4200
    pub vblank_nmi_en: bool,
    pub hv_timer_irq_mode: HVTimerIRQ,
    pub joypad_autoread_en: bool,
    
    // $4207-$4208
    pub h_counter_target: u16,
    // $4209-$420A
    pub v_counter_target: u16,
    
    // $4210
    pub vblank_nmi_flag: bool,
    
    // $4211
    pub hv_timer_irq_flag: bool,
    
    // $4212
    pub vblank_flag: bool,
    pub hblank_flag: bool,
    pub joypad_autoread_flag: bool,
    
    // $4213
    pub rdio: u8,
}