use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};

use snemcore::controller::{ControllerPlayer, JoypadButton, SnemController};

use crate::{
    app::messages::{MessageKind, MessageQueue},
    app::settings::{InputSource, RemapAxis, RemapButton, Settings, SnesInput},
};

/// Analog stick movement past this magnitude counts as a captured input
/// during remapping, and as a "pressed" digital direction during normal
/// play if a stick is bound to a d-pad-style SnesInput.
const AXIS_THRESHOLD: f32 = 0.5;
/// Threshold for detecting significant movement from baseline
const BUTTON_PRESS_THRESHOLD: f32 = 0.4;
/// Value considered "low" for axis-style d-pads
const BUTTON_LOW_THRESHOLD: f32 = 0.2;
/// Value considered "high" for normal buttons or axis-style high direction
const BUTTON_HIGH_THRESHOLD: f32 = 0.8;

/// A connected gamepad, surfaced to the settings UI so it can list and
/// select controllers without touching gilrs types directly.
pub struct ConnectedControllerInfo {
    pub id: GamepadId,
    pub name: String,
    pub uuid_key: String,
    pub assigned_player: Option<ControllerPlayer>,
}

/// Owns the gilrs context, maps connected gamepads to SNES player slots (0
/// and 1), and handles the "press a button to bind it" remap capture flow
/// used by the Controls settings tab.
pub struct ControllerManager {
    gilrs: Gilrs,
    p1_controller: Option<GamepadId>,
    p2_controller: Option<GamepadId>,
    connected_gamepads: Vec<GamepadId>,
    pending_remap: Option<(GamepadId, SnesInput)>,
    /// Baseline button values captured when remap begins, used to detect
    /// axis-style d-pads vs normal buttons
    remap_baselines: std::collections::HashMap<Button, f32>,
}

impl ControllerManager {
    pub fn new() -> anyhow::Result<Self> {
        let gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(Self {
            gilrs,
            p1_controller: None,
            p2_controller: None,
            connected_gamepads: Vec::new(),
            pending_remap: None,
            remap_baselines: std::collections::HashMap::new(),
        })
    }

    pub fn init_controllers(&mut self, settings: &Settings, message_queue: &mut MessageQueue) {
        // Log what mapping source each controller is using
        for (_id, gamepad) in self.gilrs.gamepads() {
            let mapping_source = gamepad.mapping_source();
            log::info!(
                "Controller '{}' (UUID: {:?}) mapping: {:?}",
                gamepad.name(),
                gamepad.uuid(),
                mapping_source
            );
        }

        let gamepads = Vec::from_iter(self.gilrs.gamepads().map(|(id, _gamepad)| id));

        for id in gamepads {
            self.connected_gamepads.push(id);
            self.assign_to_free_slot(id, settings, message_queue);
        }
    }

    /// Call once per frame, before reading button state. Drains gilrs's
    /// internal event queue (cheap -- no blocking I/O happens here), keeps
    /// slot assignments in sync with connect/disconnect events, resolves
    /// any pending remap capture, and feeds current button state into the
    /// emulator core using each connected controller's saved binding.
    pub fn update(
        &mut self,
        core: &mut snemcore::Snemulator,
        settings: &mut Settings,
        message_queue: &mut MessageQueue,
    ) {
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::Connected => {
                    self.connected_gamepads.push(id);
                    self.assign_to_free_slot(id, settings, message_queue);
                }
                EventType::Disconnected => {
                    self.connected_gamepads.retain(|&gid| gid != id);
                    self.free_slot(id, message_queue);
                    if self.pending_remap.is_some_and(|(pending_id, _)| pending_id == id) {
                        self.pending_remap = None;
                        self.remap_baselines.clear();
                    }
                }
                EventType::ButtonPressed(button, _) => {
                    self.try_capture_button(id, button, settings, message_queue);
                }
                EventType::ButtonChanged(button, value, _) => {
                    // Some controllers report buttons as ButtonChanged instead of ButtonPressed
                    self.try_capture_button_changed(id, button, value, settings, message_queue);
                }
                EventType::AxisChanged(axis, value, _) => {
                    self.try_capture_axis(id, axis, value, settings, message_queue);
                }
                _ => {}
            }
        }

        if let Some(p1_buttons) = self.buttons(ControllerPlayer::Player1, settings) {
            core.p1_controller = p1_buttons;
        }

        if let Some(p2_buttons) = self.buttons(ControllerPlayer::Player2, settings) {
            core.p2_controller = p2_buttons;
        }
    }

    /// All currently connected gamepads, for the Controls settings tab.
    pub fn connected_controllers(&self) -> Vec<ConnectedControllerInfo> {
        self.connected_gamepads
            .iter()
            .map(|&id| {
                let gamepad = self.gilrs.gamepad(id);
                ConnectedControllerInfo {
                    id,
                    name: gamepad.name().to_string(),
                    uuid_key: uuid_key(gamepad.uuid()),
                    assigned_player: self.slot_of(id),
                }
            })
            .collect()
    }

    pub fn slot_of(&self, id: GamepadId) -> Option<ControllerPlayer> {
        if self.p1_controller == Some(id) {
            Some(ControllerPlayer::Player1)
        } else if self.p2_controller == Some(id) {
            Some(ControllerPlayer::Player2)
        } else {
            None
        }
    }

    /// Explicitly assigns a connected gamepad to a player slot, swapping
    /// with whatever was already there (if anything), and remembers this
    /// controller as the preferred one for that slot on future launches.
    pub fn assign_player(&mut self, id: GamepadId, player: ControllerPlayer, settings: &mut Settings) {
        if self.p1_controller == Some(id) {
            self.p1_controller = None;
        }
        if self.p2_controller == Some(id) {
            self.p2_controller = None;
        }

        let key = uuid_key(self.gilrs.gamepad(id).uuid());
        match player {
            ControllerPlayer::Player1 => {
                self.p1_controller = Some(id);
                settings.preferred_p1 = Some(key);
            }
            ControllerPlayer::Player2 => {
                self.p2_controller = Some(id);
                settings.preferred_p2 = Some(key);
            }
        }
        settings.save();
    }

    /// Starts listening for the next button press or axis movement on
    /// `id`, to bind it to `input`. Call `capturing_for` each frame to
    /// drive the "Press a button..." prompt, and `cancel_remap` to abort.
    pub fn begin_remap(&mut self, id: GamepadId, input: SnesInput) {
        self.pending_remap = Some((id, input));
        
        // Snapshot current button values as baselines
        self.remap_baselines.clear();
        let gamepad = self.gilrs.gamepad(id);
        
        for button in [
            Button::DPadUp, Button::DPadDown, Button::DPadLeft, Button::DPadRight,
            Button::South, Button::East, Button::North, Button::West,
            Button::LeftTrigger, Button::LeftTrigger2,
            Button::RightTrigger, Button::RightTrigger2,
            Button::Select, Button::Start,
            Button::LeftThumb, Button::RightThumb,
        ] {
            if let Some(data) = gamepad.button_data(button) {
                self.remap_baselines.insert(button, data.value());
            }
        }
    }

    pub fn cancel_remap(&mut self) {
        self.pending_remap = None;
        self.remap_baselines.clear();
    }

    /// If a remap capture is in progress for `id`, returns which SNES
    /// input it's waiting to bind.
    pub fn capturing_for(&self, id: GamepadId) -> Option<SnesInput> {
        self.pending_remap
            .and_then(|(pending_id, input)| (pending_id == id).then_some(input))
    }

    fn try_capture_button(
        &mut self,
        id: GamepadId,
        button: Button,
        settings: &mut Settings,
        message_queue: &mut MessageQueue,
    ) {
        let Some((pending_id, input)) = self.pending_remap else { return };

        if pending_id != id {
            return;
        }

        let Some(remap_button) = to_remap_button(button) else { return };

        self.commit_binding(id, input, InputSource::Button(remap_button), settings, message_queue);
    }

    fn try_capture_button_changed(
        &mut self,
        id: GamepadId,
        button: Button,
        value: f32,
        settings: &mut Settings,
        message_queue: &mut MessageQueue,
    ) {
        let Some((pending_id, input)) = self.pending_remap else { return };

        if pending_id != id {
            return;
        }

        let Some(remap_button) = to_remap_button(button) else { return };
        let baseline = self.remap_baselines.get(&button).copied().unwrap_or(0.0);
        
        // Determine if this is a meaningful press based on movement from baseline
        let source = if baseline > 0.3 && baseline < 0.7 {
            // Baseline is near middle → axis-style d-pad
            if value < BUTTON_LOW_THRESHOLD {
                // Moved toward 0 = "low" direction (e.g., Up on vertical axis)
                InputSource::ButtonLow(remap_button)
            } else if value > BUTTON_HIGH_THRESHOLD {
                // Moved toward 1 = "high" direction (e.g., Down on vertical axis)
                InputSource::Button(remap_button)
            } else {
                return; // Still near neutral, ignore
            }
        } else {
            // Baseline is near 0 → normal digital button
            if value > BUTTON_HIGH_THRESHOLD {
                InputSource::Button(remap_button)
            } else {
                return; // Release or noise, ignore
            }
        };
        
        self.commit_binding(id, input, source, settings, message_queue);
    }

    fn try_capture_axis(
        &mut self,
        id: GamepadId,
        axis: Axis,
        value: f32,
        settings: &mut Settings,
        message_queue: &mut MessageQueue,
    ) {
        let Some((pending_id, input)) = self.pending_remap else { return };
        if pending_id != id || value.abs() < AXIS_THRESHOLD {
            return;
        }
        let Some(remap_axis) = to_remap_axis(axis) else { return };

        let source = if value > 0.0 {
            InputSource::AxisPositive(remap_axis)
        } else {
            InputSource::AxisNegative(remap_axis)
        };

        self.commit_binding(id, input, source, settings, message_queue);
    }

    fn commit_binding(
        &mut self,
        id: GamepadId,
        input: SnesInput,
        source: InputSource,
        settings: &mut Settings,
        message_queue: &mut MessageQueue,
    ) {
        let gamepad = self.gilrs.gamepad(id);
        let key = uuid_key(gamepad.uuid());
        let name = gamepad.name().to_string();

        settings.set_binding(&key, &name, input, source);
        self.pending_remap = None;
        self.remap_baselines.clear();

        message_queue.push(
            MessageKind::Info,
            format!("Bound {} to {}", input.label(), source.label()),
            std::time::Duration::from_secs_f32(2.0),
            Some(log::Level::Debug),
        );
    }

    fn assign_to_free_slot(&mut self, id: GamepadId, settings: &Settings, message_queue: &mut MessageQueue) {
        let key = uuid_key(self.gilrs.gamepad(id).uuid());
        let wants_p1 = settings.preferred_p1.as_deref() == Some(key.as_str());
        let wants_p2 = settings.preferred_p2.as_deref() == Some(key.as_str());

        if wants_p1 {
            message_queue.push(
                MessageKind::Info,
                format!("Restored gamepad {} as Player 1", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs_f32(3.0),
                Some(log::Level::Debug),
            );

            if self.p2_controller.is_none() {
                self.p2_controller = self.p1_controller;

                if let Some(p2_id) = self.p2_controller {
                    message_queue.push(
                        MessageKind::Info,
                        format!("Set gamepad {} as Player 2", self.gilrs.gamepad(p2_id).name()),
                        std::time::Duration::from_secs_f32(3.0),
                        Some(log::Level::Debug),
                    );
                }
            }

            self.p1_controller = Some(id);
        } else if wants_p2 {
            message_queue.push(
                MessageKind::Info,
                format!("Restored gamepad {} as Player 2", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs_f32(3.0),
                Some(log::Level::Debug),
            );

            if self.p1_controller.is_none() {
                self.p1_controller = self.p2_controller;

                if let Some(p1_id) = self.p1_controller {
                    message_queue.push(
                        MessageKind::Info,
                        format!("Set gamepad {} as Player 1", self.gilrs.gamepad(p1_id).name()),
                        std::time::Duration::from_secs_f32(3.0),
                        Some(log::Level::Debug),
                    );
                }
            }

            self.p2_controller = Some(id);
        } else if self.p1_controller.is_none() {
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

    /// Current SNES-mapped button state for a player, using that
    /// controller's saved (or default) binding. Returns `None` if no
    /// controller is assigned to that slot.
    fn buttons(&mut self, player: ControllerPlayer, settings: &Settings) -> Option<SnemController> {
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

        let key = uuid_key(gamepad.uuid());
        let binding = settings.binding_for(&key, gamepad.name());

        let mut controller = SnemController::default();
        for (input, joypad_button) in [
            (SnesInput::Up, JoypadButton::Up),
            (SnesInput::Down, JoypadButton::Down),
            (SnesInput::Left, JoypadButton::Left),
            (SnesInput::Right, JoypadButton::Right),
            (SnesInput::A, JoypadButton::A),
            (SnesInput::B, JoypadButton::B),
            (SnesInput::X, JoypadButton::X),
            (SnesInput::Y, JoypadButton::Y),
            (SnesInput::L, JoypadButton::L1),
            (SnesInput::R, JoypadButton::R1),
            (SnesInput::Start, JoypadButton::Start),
            (SnesInput::Select, JoypadButton::Select),
        ] {
            let pressed = binding
                .bindings
                .get(&input)
                .is_some_and(|source| is_source_active(&gamepad, *source));
            controller.set_button(joypad_button, pressed);
        }

        Some(controller)
    }
}

fn is_source_active(gamepad: &gilrs::Gamepad, source: InputSource) -> bool {
    match source {
        InputSource::Button(remap_button) => {
            let btn = from_remap_button(remap_button);
            match gamepad.button_data(btn) {
                Some(data) => data.value() > BUTTON_HIGH_THRESHOLD,
                None => gamepad.is_pressed(btn),
            }
        }
        InputSource::ButtonLow(remap_button) => {
            let btn = from_remap_button(remap_button);
            gamepad.button_data(btn)
                .map(|d| d.value() < BUTTON_LOW_THRESHOLD)
                .unwrap_or(false)
        }
        InputSource::AxisPositive(remap_axis) => {
            gamepad.value(from_remap_axis(remap_axis)) > AXIS_THRESHOLD
        }
        InputSource::AxisNegative(remap_axis) => {
            gamepad.value(from_remap_axis(remap_axis)) < -AXIS_THRESHOLD
        }
    }
}

fn uuid_key(uuid: [u8; 16]) -> String {
    uuid.iter().map(|b| format!("{b:02x}")).collect()
}

fn to_remap_button(button: Button) -> Option<RemapButton> {
    Some(match button {
        Button::South => RemapButton::South,
        Button::East => RemapButton::East,
        Button::North => RemapButton::North,
        Button::West => RemapButton::West,
        Button::LeftTrigger => RemapButton::LeftTrigger,
        Button::LeftTrigger2 => RemapButton::LeftTrigger2,
        Button::RightTrigger => RemapButton::RightTrigger,
        Button::RightTrigger2 => RemapButton::RightTrigger2,
        Button::Select => RemapButton::Select,
        Button::Start => RemapButton::Start,
        Button::DPadUp => RemapButton::DPadUp,
        Button::DPadDown => RemapButton::DPadDown,
        Button::DPadLeft => RemapButton::DPadLeft,
        Button::DPadRight => RemapButton::DPadRight,
        Button::LeftThumb => RemapButton::LeftThumb,
        Button::RightThumb => RemapButton::RightThumb,
        // Mode, C, Z, Unknown -- not exposed for binding.
        _ => return None,
    })
}

fn from_remap_button(button: RemapButton) -> Button {
    match button {
        RemapButton::South => Button::South,
        RemapButton::East => Button::East,
        RemapButton::North => Button::North,
        RemapButton::West => Button::West,
        RemapButton::LeftTrigger => Button::LeftTrigger,
        RemapButton::LeftTrigger2 => Button::LeftTrigger2,
        RemapButton::RightTrigger => Button::RightTrigger,
        RemapButton::RightTrigger2 => Button::RightTrigger2,
        RemapButton::Select => Button::Select,
        RemapButton::Start => Button::Start,
        RemapButton::DPadUp => Button::DPadUp,
        RemapButton::DPadDown => Button::DPadDown,
        RemapButton::DPadLeft => Button::DPadLeft,
        RemapButton::DPadRight => Button::DPadRight,
        RemapButton::LeftThumb => Button::LeftThumb,
        RemapButton::RightThumb => Button::RightThumb,
    }
}

fn to_remap_axis(axis: Axis) -> Option<RemapAxis> {
    Some(match axis {
        Axis::LeftStickX => RemapAxis::LeftStickX,
        Axis::LeftStickY => RemapAxis::LeftStickY,
        Axis::RightStickX => RemapAxis::RightStickX,
        Axis::RightStickY => RemapAxis::RightStickY,
        Axis::LeftZ => RemapAxis::LeftZ,
        Axis::RightZ => RemapAxis::RightZ,
        Axis::DPadX => RemapAxis::DPadX,
        Axis::DPadY => RemapAxis::DPadY,
        _ => return None,
    })
}

fn from_remap_axis(axis: RemapAxis) -> Axis {
    match axis {
        RemapAxis::LeftStickX => Axis::LeftStickX,
        RemapAxis::LeftStickY => Axis::LeftStickY,
        RemapAxis::RightStickX => Axis::RightStickX,
        RemapAxis::RightStickY => Axis::RightStickY,
        RemapAxis::LeftZ => Axis::LeftZ,
        RemapAxis::RightZ => Axis::RightZ,
        RemapAxis::DPadX => Axis::DPadX,
        RemapAxis::DPadY => Axis::DPadY,
    }
}