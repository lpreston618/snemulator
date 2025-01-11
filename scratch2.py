a == """if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = Cpu65c816::ONE_CYCLE;
                } else {
                    extra_clocks = Cpu65c816::TWO_CYCLE;
                }"""