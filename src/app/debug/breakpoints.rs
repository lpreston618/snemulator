#[derive(Clone, Copy)]
pub struct BreakpointInfo {
    pub addr: u32,
    pub force_m: bool,
    pub force_x: bool,
    pub force_e: bool,
}

impl BreakpointInfo {
    pub fn new(addr: u32) -> Self {
        Self {
            addr,
            force_m: false,
            force_x: false,
            force_e: false,
        }
    }
}

impl std::hash::Hash for BreakpointInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

impl PartialEq for BreakpointInfo {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Eq for BreakpointInfo {}