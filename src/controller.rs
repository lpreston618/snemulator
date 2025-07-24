use libretro_rs::retro::device::JoypadButton;

// #[derive(Debug)]
// pub enum JoypadButton {
//     A = 0,
//     B = 1,
//     X = 2,
//     Y = 3,
//     Up = 4,
//     Down = 5,
//     Left = 6,
//     Right = 7,
//     Select = 8,
//     Start = 9,
//     L = 10,
//     R = 11,
// }

// impl Into<retro::device::JoypadButton> for JoypadButton {
//     fn into(self) -> retro::device::JoypadButton {
//         match self {
//             JoypadButton::A => retro::device::JoypadButton::A,
//             JoypadButton::B => retro::device::JoypadButton::B,
//             JoypadButton::X => retro::device::JoypadButton::X,
//             JoypadButton::Y => retro::device::JoypadButton::Y,
//             JoypadButton::Up => retro::device::JoypadButton::Up,
//             JoypadButton::Down => retro::device::JoypadButton::Down,
//             JoypadButton::Left => retro::device::JoypadButton::Left,
//             JoypadButton::Right => retro::device::JoypadButton::Right,
//             JoypadButton::Select => retro::device::JoypadButton::Select,
//             JoypadButton::Start => retro::device::JoypadButton::Start,
//             JoypadButton::L => retro::device::JoypadButton::L1,
//             JoypadButton::R => retro::device::JoypadButton::R1,
//         }
//     }
// }

pub struct SnemController {
    buttons: [bool; 12],
}

impl SnemController {
    pub fn new() -> SnemController {
        SnemController { buttons: [false; 12] }
    }

    pub fn is_button_pressed(&self, button: JoypadButton) -> bool {
        self.buttons[button as usize]
    }

    pub fn update_button_state(&mut self, button: JoypadButton, pressed: bool) {
        self.buttons[button as usize] = pressed;
    }

    pub fn state_as_u16(&mut self) -> u16 {
        macro_rules! button_pressed_bit {
            ($button:expr) => {
                if self.is_button_pressed($button) { 1 << ($button as usize) } else { 0 }
            };
        }

        // Put into order SNES expects to read it in
        let b =      button_pressed_bit!(JoypadButton::B);
        let y =      button_pressed_bit!(JoypadButton::Y);
        let select = button_pressed_bit!(JoypadButton::Select);
        let start =  button_pressed_bit!(JoypadButton::Start);
        let up =     button_pressed_bit!(JoypadButton::Up);
        let down =   button_pressed_bit!(JoypadButton::Down);
        let left =   button_pressed_bit!(JoypadButton::Left);
        let right =  button_pressed_bit!(JoypadButton::Right);
        let a =      button_pressed_bit!(JoypadButton::A);
        let x =      button_pressed_bit!(JoypadButton::X);
        let l =      button_pressed_bit!(JoypadButton::L1);
        let r =      button_pressed_bit!(JoypadButton::R1);

        b | y | select | start | up | down | left | right | a | x | l | r
    }
}