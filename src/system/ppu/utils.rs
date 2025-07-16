use std::cell::Cell;

pub(super) trait SetCellBytes {
    fn set_hi(&self, data: u8);
    fn set_lo(&self, data: u8);
}

impl SetCellBytes for Cell<u16> {
    fn set_hi(&self, data: u8) {
        self.set((self.get() & 0x00FF) | ((data as u16) << 8));
    }
    fn set_lo(&self, data: u8) {
        self.set((self.get() & 0xFF00) | (data as u16));
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) enum ToggleState {
    #[default]
    LoByte,
    HiByte,
}

pub(super) trait Togglable {
    /// Returns a bool reporting whether the latch state is high.
    fn is_high(&self) -> bool;
    /// Toggles the latch and returns a bool reporting whether the latch was high
    /// BEFORE the toggle.
    fn toggle(&self) -> bool;
    /// Sets the toggle state to low/0.
    fn set_lo(&self);
    /// Sets the toggle state to high/1.
    fn set_hi(&self);
}

impl Togglable for Cell<ToggleState> {
    fn is_high(&self) -> bool { self.get() == ToggleState::HiByte }
    fn toggle(&self) -> bool {
        self.replace(
            match self.get() {
                ToggleState::LoByte => ToggleState::HiByte,
                ToggleState::HiByte => ToggleState::LoByte,
            }
        ) == ToggleState::HiByte
    }
    fn set_lo(&self) { self.set(ToggleState::LoByte); }
    fn set_hi(&self) { self.set(ToggleState::HiByte); }
}