pub struct Timer<const PERIOD: usize> {
    enable: bool,
    target: u8,
    counter: u8,
    internal_counter: u8,

    clocks: usize,
}

impl<const PERIOD: usize> Timer<PERIOD> {
    pub fn new() -> Timer<PERIOD> {
        Timer {
            enable: false,
            target: 0,
            counter: 0,
            internal_counter: 0,
            clocks: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enable = false;
        self.target = 0xFF;
        self.counter = 0;
        self.internal_counter = 0;
        self.clocks = 0;
    }

    pub fn clock(&mut self) {
        if !self.enable {
            self.counter = 0;
            return;
        }

        self.clocks += 1;

        if self.clocks == PERIOD {
            self.clocks = 0;
            self.internal_counter += 1;

            if self.internal_counter == self.target {
                self.internal_counter = 0;
                self.counter += 1;
                self.counter &= 0xF;
            }
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        if !self.enable && enable {
            self.counter = 0;
            self.internal_counter = 0;
        }

        self.enable = enable;
    }

    pub fn set_target(&mut self, data: u8) { self.target = data; }
    pub fn get_counter(&mut self) -> u8 {
        let data = self.counter;
        self.counter = 0;
        data
    }
}