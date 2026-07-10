use gilrs::{Button, Event, EventType, Gilrs, GamepadId};

use snemcore::controller::{ControllerPlayer, JoypadButton, SnemController};

use crate::app::messages::{MessageKind, MessageQueue};

/// Owns the gilrs context and maps connected gamepads to SNES player slots (0 and 1).
pub struct ControllerManager {
    gilrs: Gilrs,
    p1_controller: Option<GamepadId>,
    p2_controller: Option<GamepadId>,
    p1_custom_mapping: Option<gilrs::Mapping>,
    p2_custom_mapping: Option<gilrs::Mapping>,
    connected_gamepads: Vec<GamepadId>,
}

impl ControllerManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            gilrs: Gilrs::new().map_err(|e| anyhow::anyhow!("{e}"))?,
            p1_controller: None,
            p2_controller: None,
            p1_custom_mapping: None,
            p2_custom_mapping: None,
            connected_gamepads: Vec::new(),
        })
    }

    pub fn init_controllers(&mut self, message_queue: &mut MessageQueue) {
        let gamepads = Vec::from_iter(self.gilrs.gamepads().map(|(id, _gamepad)| id));

        for id in gamepads {
            self.connected_gamepads.push(id);

            self.assign_to_free_slot(id, message_queue);
        }
    }

    /// Call once per frame, before reading button state. This just drains
    /// gilrs's internal event queue (cheap -- no blocking I/O happens here)
    /// and keeps slot assignments in sync with connect/disconnect events.
    pub fn update(&mut self, core: &mut snemcore::Snemulator, message_queue: &mut MessageQueue) {
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::Connected => {
                    self.connected_gamepads.push(id);
                    self.assign_to_free_slot(id, message_queue);
                },
                EventType::Disconnected => {
                    self.connected_gamepads.retain(|&gid| gid != id);
                    self.free_slot(id, message_queue);
                },
                // Button/axis state is read on demand via `buttons()`, so we
                // don't need to track individual press/release events here.
                _ => {}
            }
        }

        if let Some(p1_buttons) = self.buttons(ControllerPlayer::Player1) {
            core.p1_controller = p1_buttons;
        }

        if let Some(p2_buttons) = self.buttons(ControllerPlayer::Player2) {
            core.p2_controller = p2_buttons;
        }
    }

    fn assign_to_free_slot(&mut self, id: GamepadId, message_queue: &mut MessageQueue) {
        if self.p1_controller.is_none() {
            self.p1_controller = Some(id);

            message_queue.push(
                MessageKind::Info,
                format!("Added gamepad {} as Player 1", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs_f32(3.0),
                Some(log::Level::Debug),
            );
        } else if self.p2_controller.is_none() {
            self.p2_controller = Some(id);

            message_queue.push(
                MessageKind::Info,
                format!("Added gamepad {} as Player 2", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs_f32(3.0),
                Some(log::Level::Debug),
            );
        }
    }

    fn free_slot(&mut self, id: GamepadId, message_queue: &mut MessageQueue) {
        if Some(id) == self.p1_controller {
            self.p1_controller = None;

            message_queue.push(
                MessageKind::Info,
                format!("Gamepad {} disconnected from Player 1", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs_f32(3.0),
                Some(log::Level::Debug),
            );
        } else if Some(id) == self.p2_controller {
            self.p2_controller = None;

            message_queue.push(
                MessageKind::Info,
                format!("Gamepad {} disconnected from Player 2", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs_f32(3.0),
                Some(log::Level::Debug),
            );
        }
    }

    /// Current SNES-mapped button state for a player (0 or 1).
    /// Returns all-false if no controller is assigned to that slot.
    fn buttons(&mut self, player: ControllerPlayer) -> Option<SnemController> {
        let id = match player {
            ControllerPlayer::Player1 => self.p1_controller,
            ControllerPlayer::Player2 => self.p2_controller,
        }?;

        let gamepad = self.gilrs.gamepad(id);

        // If the gamepad is disconnected, remove it from the list of connected gamepads and free the slot.
        if !gamepad.is_connected() {
            self.connected_gamepads.retain(|&gid| gid != id);
            
            match player {
                ControllerPlayer::Player1 => self.p1_controller = None,
                ControllerPlayer::Player2 => self.p2_controller = None,
            }

            return None;
        }

        let mut controller = SnemController::default();
        controller.set_button(JoypadButton::B, gamepad.is_pressed(Button::South));
        controller.set_button(JoypadButton::A, gamepad.is_pressed(Button::East));
        controller.set_button(JoypadButton::Y, gamepad.is_pressed(Button::West));
        controller.set_button(JoypadButton::X, gamepad.is_pressed(Button::North));
        controller.set_button(JoypadButton::L1, gamepad.is_pressed(Button::LeftTrigger));
        controller.set_button(JoypadButton::R1, gamepad.is_pressed(Button::RightTrigger));
        controller.set_button(JoypadButton::Select, gamepad.is_pressed(Button::Select));
        controller.set_button(JoypadButton::Start, gamepad.is_pressed(Button::Start));
        controller.set_button(JoypadButton::Up, gamepad.is_pressed(Button::DPadUp));
        controller.set_button(JoypadButton::Down, gamepad.is_pressed(Button::DPadDown));
        controller.set_button(JoypadButton::Left, gamepad.is_pressed(Button::DPadLeft));
        controller.set_button(JoypadButton::Right, gamepad.is_pressed(Button::DPadRight));
        Some(controller)
    }

    pub fn apply_custom_mapping(
        &mut self,
        player: ControllerPlayer,
        mapping: gilrs::Mapping,
    ) -> anyhow::Result<()> {
        let (id, player_label) = match player {
            ControllerPlayer::Player1 => (self.p1_controller, "Player 1"),
            ControllerPlayer::Player2 => (self.p2_controller, "Player 2"),
        };

        let id = id.ok_or_else(|| anyhow::anyhow!("no gamepad assigned to {player_label}"))?;

        self.gilrs
            .set_mapping(id.into(), &mapping, None)
            .map_err(|e| anyhow::anyhow!("failed to set mapping: {e}"))?;

        match player {
            ControllerPlayer::Player1 => self.p1_custom_mapping = Some(mapping),
            ControllerPlayer::Player2 => self.p2_custom_mapping = Some(mapping),
        }

        Ok(())
    }
}

// Example usage in your main loop:
//
// let mut controllers = ControllerManager::new().expect("failed to init gilrs");
//
// loop {
//     controllers.update();
//     let p1 = controllers.buttons(0);
//     let p2 = controllers.buttons(1);
//     // feed p1.to_register() / p2.to_register() into your PPU/joypad state
// }