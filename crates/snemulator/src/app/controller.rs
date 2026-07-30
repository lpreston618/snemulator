use std::collections::HashMap;

use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};

// NOTE: adjust this import to match wherever your SDL3 bindings expose
// KeyboardState / Scancode (e.g. `sdl3::keyboard` if you're using the
// `sdl3` crate).
use sdl3::keyboard::{KeyboardState, Scancode};

use snemcore::controller::{ControllerPlayer, JoypadButton, SnemController};

use crate::{
    app::messages::{MessageKind, MessageQueue},
    app::settings::{InputSource, RemapAxis, RemapButton, Settings, SnesInput},
};

/// Analog stick movement past this magnitude counts as a captured input
/// during remapping, and as a "pressed" digital direction during normal
/// play if a stick is bound to a d-pad-style SnesInput.
const AXIS_THRESHOLD: f32 = 0.5;
/// Value considered "low" for axis-style d-pads
const BUTTON_LOW_THRESHOLD: f32 = 0.2;
/// Value considered "high" for normal buttons or axis-style high direction
const BUTTON_HIGH_THRESHOLD: f32 = 0.8;

/// The (SnesInput, JoypadButton) pairs read every frame. Shared between the
/// gamepad and keyboard button-resolution paths so they stay in sync.
const INPUT_BUTTON_PAIRS: [(SnesInput, JoypadButton); 12] = [
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
];

/// The input device backing a player slot. A player can be driven by the
/// keyboard or by a specific connected gamepad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerInputDevice {
    Keyboard,
    Gamepad(GamepadId),
}

/// A connected gamepad, surfaced to the settings UI so it can list and
/// select controllers without touching gilrs types directly.
pub struct ConnectedControllerInfo {
    pub id: GamepadId,
    pub name: String,
    pub uuid_key: String,
    // pub assigned_player: Option<ControllerPlayer>,
}

/// Owns the gilrs context, maps connected gamepads (or the keyboard) to
/// SNES player slots (0 and 1), and handles the "press a button to bind
/// it" remap capture flow used by the Controls settings tab.
///
/// Player 1 defaults to the keyboard, matching the previous hardcoded
/// behavior. Either slot can be reassigned to the keyboard or to any
/// connected gamepad via `assign_player`.
pub struct ControllerManager {
    gilrs: Gilrs,
    p1_device: Option<PlayerInputDevice>,
    p2_device: Option<PlayerInputDevice>,
    connected_gamepads: Vec<GamepadId>,
    pending_remap: Option<(GamepadId, SnesInput)>,
    /// Baseline button values captured when remap begins, used to detect
    /// axis-style d-pads vs normal buttons
    remap_baselines: HashMap<Button, f32>,
    /// SNES input currently waiting for the next keyboard press to bind
    /// it, set by `begin_keyboard_remap` and resolved by
    /// `try_capture_keyboard`.
    pending_keyboard_remap: Option<SnesInput>,
}

impl ControllerManager {
    pub fn new() -> anyhow::Result<Self> {
        let gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(Self {
            gilrs,
            p1_device: Some(PlayerInputDevice::Keyboard),
            p2_device: None,
            connected_gamepads: Vec::new(),
            pending_remap: None,
            remap_baselines: HashMap::new(),
            pending_keyboard_remap: None,
        })
    }

    pub fn init_controllers(&mut self, settings: &mut Settings, message_queue: &mut MessageQueue) {
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

        if Some("keyboard".to_string()) == settings.preferred_p1 {
            self.assign_player(PlayerInputDevice::Keyboard, ControllerPlayer::Player1, settings);
        } else if Some("keyboard".to_string()) == settings.preferred_p2 {
            self.assign_player(PlayerInputDevice::Keyboard, ControllerPlayer::Player2, settings);
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
    /// emulator core using each slot's assigned device (keyboard or
    /// gamepad) and saved binding.
    ///
    /// `keyboard` is a per-frame snapshot of keyboard state -- e.g. from
    /// `event_pump.keyboard_state()` in your SDL3 main loop. Grab it fresh
    /// each frame and pass it in here; ControllerManager does not own an
    /// SDL3 event pump itself.
    pub fn update(
        &mut self,
        core: &mut snemcore::Snemulator,
        settings: &mut Settings,
        message_queue: &mut MessageQueue,
        keyboard: &KeyboardState,
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
                    self.try_capture_button(id, button, settings);
                }
                EventType::ButtonChanged(button, value, _) => {
                    // Some controllers report buttons as ButtonChanged instead of ButtonPressed
                    self.try_capture_button_changed(id, button, value, settings);
                }
                EventType::AxisChanged(axis, value, _) => {
                    self.try_capture_axis(id, axis, value, settings);
                }
                _ => {}
            }
        }

        if let Some(p1_buttons) = self.buttons(ControllerPlayer::Player1, settings, keyboard) {
            core.p1_controller = p1_buttons;
        }

        if let Some(p2_buttons) = self.buttons(ControllerPlayer::Player2, settings, keyboard) {
            core.p2_controller = p2_buttons;
        }
    }

    /// All currently connected gamepads, for the Controls settings tab.
    /// (The keyboard is always "connected" and isn't listed here -- surface
    /// it as a separate, permanently-available option in the UI.)
    pub fn connected_controllers(&self) -> Vec<ConnectedControllerInfo> {
        self.connected_gamepads
            .iter()
            .map(|&id| {
                let gamepad = self.gilrs.gamepad(id);
                ConnectedControllerInfo {
                    id,
                    name: gamepad.name().to_string(),
                    uuid_key: uuid_key(gamepad.uuid()),
                    // assigned_player: self.slot_of(id),
                }
            })
            .collect()
    }

    /// Which player slot (if any) a specific gamepad is currently driving.
    // pub fn slot_of(&self, id: GamepadId) -> Option<ControllerPlayer> {
    //     let device = PlayerInputDevice::Gamepad(id);
    //     if self.p1_device == Some(device) {
    //         Some(ControllerPlayer::Player1)
    //     } else if self.p2_device == Some(device) {
    //         Some(ControllerPlayer::Player2)
    //     } else {
    //         None
    //     }
    // }

    /// The device (keyboard or gamepad) currently assigned to a player
    /// slot, for the Controls settings tab.
    pub fn device_for(&self, player: ControllerPlayer) -> Option<PlayerInputDevice> {
        match player {
            ControllerPlayer::Player1 => self.p1_device,
            ControllerPlayer::Player2 => self.p2_device,
        }
    }

    /// Explicitly assigns a device (keyboard or a connected gamepad) to a
    /// player slot, swapping with whatever already had that device (if
    /// anything), and remembers a gamepad choice as the preferred one for
    /// that slot on future launches. Assigning the keyboard clears any
    /// remembered gamepad preference for that slot.
    pub fn assign_player(&mut self, device: PlayerInputDevice, player: ControllerPlayer, settings: &mut Settings) {
        if self.p1_device == Some(device) {
            self.p1_device = None;
        }
        if self.p2_device == Some(device) {
            self.p2_device = None;
        }

        let preferred = match device {
            PlayerInputDevice::Gamepad(id) => Some(uuid_key(self.gilrs.gamepad(id).uuid())),
            PlayerInputDevice::Keyboard => Some("keyboard".to_string()),
        };

        match player {
            ControllerPlayer::Player1 => {
                self.p1_device = Some(device);
                settings.preferred_p1 = preferred;
            }
            ControllerPlayer::Player2 => {
                self.p2_device = Some(device);
                settings.preferred_p2 = preferred;
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

    /// Starts listening for the next keyboard press, to bind it to
    /// `input`. Call `keyboard_capturing` each frame to drive the "Press a
    /// key..." prompt, and `cancel_keyboard_remap` to abort.
    pub fn begin_keyboard_remap(&mut self, input: SnesInput) {
        self.pending_keyboard_remap = Some(input);
    }

    pub fn cancel_keyboard_remap(&mut self) {
        self.pending_keyboard_remap = None;
    }

    /// If a keyboard remap capture is in progress, returns which SNES
    /// input it's waiting to bind.
    pub fn keyboard_capturing(&self) -> Option<SnesInput> {
        self.pending_keyboard_remap
    }

    /// If a keyboard remap capture is pending, binds `keycode` to the waiting
    /// SnesInput and clears the capture.
    pub fn try_capture_keyboard(&mut self, scancode: Scancode, settings: &mut Settings) {
        let Some(input) = self.pending_keyboard_remap else { return };

        settings.keyboard_bindings.insert(input, scancode.to_i32());
        settings.save();
        self.pending_keyboard_remap = None;
    }

    fn try_capture_button(
        &mut self,
        id: GamepadId,
        button: Button,
        settings: &mut Settings,
    ) {
        let Some((pending_id, input)) = self.pending_remap else { return };

        if pending_id != id {
            return;
        }

        let Some(remap_button) = to_remap_button(button) else { return };

        self.commit_binding(id, input, InputSource::Button(remap_button), settings);
    }

    fn try_capture_button_changed(
        &mut self,
        id: GamepadId,
        button: Button,
        value: f32,
        settings: &mut Settings,
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

        self.commit_binding(id, input, source, settings);
    }

    fn try_capture_axis(
        &mut self,
        id: GamepadId,
        axis: Axis,
        value: f32,
        settings: &mut Settings,
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

        self.commit_binding(id, input, source, settings);
    }

    fn commit_binding(
        &mut self,
        id: GamepadId,
        input: SnesInput,
        source: InputSource,
        settings: &mut Settings,
    ) {
        let gamepad = self.gilrs.gamepad(id);
        let key = uuid_key(gamepad.uuid());
        let name = gamepad.name().to_string();

        settings.set_binding(&key, &name, input, source);
        self.pending_remap = None;
        self.remap_baselines.clear();
    }

    fn assign_to_free_slot(&mut self, id: GamepadId, settings: &Settings, message_queue: &mut MessageQueue) {
        let key = uuid_key(self.gilrs.gamepad(id).uuid());
        let wants_p1 = settings.preferred_p1.as_deref() == Some(key.as_str());
        let wants_p2 = settings.preferred_p2.as_deref() == Some(key.as_str());
        let device = PlayerInputDevice::Gamepad(id);

        if wants_p1 {
            message_queue.push(
                MessageKind::Info,
                format!("Restored gamepad {} as Player 1", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs(3),
                Some(log::Level::Debug),
            );

            if self.p2_device.is_none() {
                // Bump whatever (keyboard or another gamepad) was on
                // Player 1 down to Player 2, rather than dropping it.
                self.p2_device = self.p1_device.take();

                if let Some(bumped) = self.p2_device {
                    message_queue.push(
                        MessageKind::Info,
                        format!("Set {} as Player 2", self.device_name(bumped)),
                        std::time::Duration::from_secs(3),
                        Some(log::Level::Debug),
                    );
                }
            }

            self.p1_device = Some(device);
        } else if wants_p2 {
            message_queue.push(
                MessageKind::Info,
                format!("Restored gamepad {} as Player 2", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs(3),
                Some(log::Level::Debug),
            );

            if self.p1_device.is_none() {
                self.p1_device = self.p2_device.take();

                if let Some(bumped) = self.p1_device {
                    message_queue.push(
                        MessageKind::Info,
                        format!("Set {} as Player 1", self.device_name(bumped)),
                        std::time::Duration::from_secs(3),
                        Some(log::Level::Debug),
                    );
                }
            }

            self.p2_device = Some(device);
        } else if self.p1_device.is_none() {
            self.p1_device = Some(device);
            message_queue.push(
                MessageKind::Info,
                format!("Added gamepad {} as Player 1", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs(3),
                Some(log::Level::Debug),
            );
        } else if self.p2_device.is_none() {
            self.p2_device = Some(device);
            message_queue.push(
                MessageKind::Info,
                format!("Added gamepad {} as Player 2", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs(3),
                Some(log::Level::Debug),
            );
        }
    }

    fn free_slot(&mut self, id: GamepadId, message_queue: &mut MessageQueue) {
        let device = PlayerInputDevice::Gamepad(id);

        if self.p1_device == Some(device) {
            self.p1_device = None;

            message_queue.push(
                MessageKind::Info,
                format!("Gamepad {} disconnected from Player 1", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs(3),
                Some(log::Level::Debug),
            );
        } else if self.p2_device == Some(device) {
            self.p2_device = None;

            message_queue.push(
                MessageKind::Info,
                format!("Gamepad {} disconnected from Player 2", self.gilrs.gamepad(id).name()),
                std::time::Duration::from_secs(3),
                Some(log::Level::Debug),
            );
        }
    }

    fn device_name(&self, device: PlayerInputDevice) -> String {
        match device {
            PlayerInputDevice::Keyboard => "Keyboard".to_string(),
            PlayerInputDevice::Gamepad(id) => self.gilrs.gamepad(id).name().to_string(),
        }
    }

    /// Current SNES-mapped button state for a player, from whichever
    /// device (keyboard or gamepad) is assigned to that slot. Returns
    /// `None` if no device is assigned to that slot.
    fn buttons(
        &mut self,
        player: ControllerPlayer,
        settings: &Settings,
        keyboard: &KeyboardState,
    ) -> Option<SnemController> {
        let device = match player {
            ControllerPlayer::Player1 => self.p1_device,
            ControllerPlayer::Player2 => self.p2_device,
        }?;

        match device {
            PlayerInputDevice::Keyboard => Some(keyboard_buttons(settings, keyboard)),
            PlayerInputDevice::Gamepad(id) => {
                let gamepad = self.gilrs.gamepad(id);

                // If the gamepad is disconnected, remove it from the list of
                // connected gamepads and free the slot.
                if !gamepad.is_connected() {
                    self.connected_gamepads.retain(|&gid| gid != id);

                    match player {
                        ControllerPlayer::Player1 => self.p1_device = None,
                        ControllerPlayer::Player2 => self.p2_device = None,
                    }

                    return None;
                }

                let key = uuid_key(gamepad.uuid());
                let binding = settings.binding_for(&key, gamepad.name());

                let mut controller = SnemController::default();
                for (input, joypad_button) in INPUT_BUTTON_PAIRS {
                    let pressed = binding
                        .bindings
                        .get(&input)
                        .is_some_and(|source| is_source_active(&gamepad, *source));
                    controller.set_button(joypad_button, pressed);
                }

                Some(controller)
            }
        }
    }

}

/// Current SNES-mapped button state read from the keyboard, using
/// `settings.keyboard_bindings` (falling back to `default_scancode_for`
/// for any SnesInput the user hasn't remapped yet -- mirrors how
/// `Settings::binding_for` falls back to `ControllerBinding::default_for`
/// for gamepads).
fn keyboard_buttons(settings: &Settings, keyboard: &KeyboardState) -> SnemController {
    let mut controller = SnemController::default();
    for (input, joypad_button) in INPUT_BUTTON_PAIRS {
        let scancode = settings
            .keyboard_bindings
            .get(&input)
            .and_then(|&code| Scancode::from_i32(code as i32))
            .unwrap_or_else(|| default_scancode_for(input));
        controller.set_button(joypad_button, keyboard.is_scancode_pressed(scancode));
    }
    controller
}

/// Fixed default keyboard layout, used for any SnesInput not present in
/// `settings.keyboard_bindings` (i.e. before the user has remapped it).
/// Public so the Controls settings tab can show the same default in its
/// "Unbound" label rather than a value that disagrees with actual runtime
/// behavior.
pub fn default_scancode_for(input: SnesInput) -> Scancode {
    match input {
        SnesInput::Up => Scancode::Up,
        SnesInput::Down => Scancode::Down,
        SnesInput::Left => Scancode::Left,
        SnesInput::Right => Scancode::Right,
        SnesInput::A => Scancode::X,
        SnesInput::B => Scancode::Z,
        SnesInput::X => Scancode::S,
        SnesInput::Y => Scancode::A,
        SnesInput::L => Scancode::Q,
        SnesInput::R => Scancode::W,
        SnesInput::Start => Scancode::Return,
        SnesInput::Select => Scancode::RShift,
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