pub mod breakpoints;
pub mod watchpoints;
pub mod ppulayers;

pub enum DebugAction {
    SingleStep,
    StepFrame,
    TogglePause,
    BreakpointHit,
    WatchpointHit,
    Reset,
    HardReset,
    None,
}