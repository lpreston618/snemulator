pub enum JoypadCmd {
    LatchJoypads,
    EnableAutoread,
    DisableAutoread,
    ClockJoy1,
    ClockJoy2,
}

#[derive(Debug, Clone, Copy)]
pub enum JoypadButton {
    B = 1 << 0,
    Y = 1 << 1,
    Select = 1 << 2,
    Start = 1 << 3,
    Up = 1 << 4,
    Down = 1 << 5,
    Left = 1 << 6,
    Right = 1 << 7,
    A = 1 << 8,
    X = 1 << 9,
    L1 = 1 << 10,
    R1 = 1 << 11,
}

#[derive(Debug, Clone, Copy)]
pub enum ControllerPlayer {
    Player1,
    Player2,
}

pub struct SnemController {
    buttons: u16,
}

impl SnemController {
    pub fn new() -> SnemController {
        SnemController { buttons: 0 }
    }

    pub fn set_button(&mut self, button: JoypadButton, pressed: bool) {
        if pressed {
            self.buttons |= button as u16;
        } else {
            self.buttons &= !(button as u16);
        }
    }
}

#[derive(Default)]
pub struct ControllerData {
    pub joy1_latch: u16,
    pub joy2_latch: u16,
    pub joy1_data1_auto: u16,
    pub joy2_data1_auto: u16,
    pub joy1_data2_auto: u16,
    pub joy2_data2_auto: u16,
    pub joypad_cmd: Option<JoypadCmd>,
}