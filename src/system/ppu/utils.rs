use std::cell::Cell;

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

/// Converts a u16 from the 0BGR0555 color format used by the SNES into the
/// libretro supported RGB565 format. The lowest bit of the green color value
/// will never be set in the RGB565 format.
pub(super) fn xbgr0555_to_rgb565(col: u16) -> u16 {
    ((col & 0x1F) << 11) | ((col & 0x03E0) << 1) | ((col & 0x7C00) >> 10)
}

/// Converts a u16 from the RGB565 color format used by libretro into the
/// SNES's color format, 0BGR0555.
pub(super) fn rgb565_to_xbgr0555(col: u16) -> u16 {
    ((col & 0x1F) << 10) | ((col & 0x07C0) >> 1) | ((col & 0xF800) >> 11)
}