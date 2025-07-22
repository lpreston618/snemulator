use libretro_rs::retro;

#[derive(Debug)]
pub enum SnemControllerButton {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
    Select = 8,
    Start = 9,
    L = 10,
    R = 11,
}

impl Into<retro::device::JoypadButton> for SnemControllerButton {
    fn into(self) -> retro::device::JoypadButton {
        match self {
            SnemControllerButton::A => retro::device::JoypadButton::A,
            SnemControllerButton::B => retro::device::JoypadButton::B,
            SnemControllerButton::X => retro::device::JoypadButton::X,
            SnemControllerButton::Y => retro::device::JoypadButton::Y,
            SnemControllerButton::Up => retro::device::JoypadButton::Up,
            SnemControllerButton::Down => retro::device::JoypadButton::Down,
            SnemControllerButton::Left => retro::device::JoypadButton::Left,
            SnemControllerButton::Right => retro::device::JoypadButton::Right,
            SnemControllerButton::Select => retro::device::JoypadButton::Select,
            SnemControllerButton::Start => retro::device::JoypadButton::Start,
            SnemControllerButton::L => retro::device::JoypadButton::L1,
            SnemControllerButton::R => retro::device::JoypadButton::R1,
        }
    }
}

pub struct SnemController {
    buttons: [bool; 12],
}

impl SnemController {
    pub fn new() -> SnemController {
        SnemController { buttons: [false; 12] }
    }

    pub fn is_button_pressed(&self, button: SnemControllerButton) -> bool {
        self.buttons[button as usize]
    }

    pub fn update_button_state(&mut self, button: SnemControllerButton, pressed: bool) {
        self.buttons[button as usize] = pressed;
    }

    pub fn state_as_u16(&mut self) -> u16 {
        // Put into order SNES expects to read it in
        let b = if self.is_button_pressed(SnemControllerButton::B)           { 1 << 0 } else { 0 };
        let y = if self.is_button_pressed(SnemControllerButton::Y)           { 1 << 1 } else { 0 };
        let select = if self.is_button_pressed(SnemControllerButton::Select) { 1 << 2 } else { 0 };
        let start = if self.is_button_pressed(SnemControllerButton::Start)   { 1 << 3 } else { 0 };
        let up = if self.is_button_pressed(SnemControllerButton::Up)         { 1 << 4 } else { 0 };
        let down = if self.is_button_pressed(SnemControllerButton::Down)     { 1 << 5 } else { 0 };
        let left = if self.is_button_pressed(SnemControllerButton::Left)     { 1 << 6 } else { 0 };
        let right = if self.is_button_pressed(SnemControllerButton::Right)   { 1 << 7 } else { 0 };
        let a = if self.is_button_pressed(SnemControllerButton::A)           { 1 << 8 } else { 0 };
        let x = if self.is_button_pressed(SnemControllerButton::X)           { 1 << 9 } else { 0 };
        let l = if self.is_button_pressed(SnemControllerButton::L)           { 1 << 10 } else { 0 };
        let r = if self.is_button_pressed(SnemControllerButton::R)           { 1 << 11 } else { 0 };

        b | y | select | start | up | down | left | right | a | x | l | r
    }
}