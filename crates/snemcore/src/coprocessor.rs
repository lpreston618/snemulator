use serde::{Deserialize, Serialize};

use crate::coprocessor::dsp::Dsp1;

pub mod dsp;

#[derive(Serialize, Deserialize, Clone)]
pub enum Coprocessor {
    // Versions of DSP-1:
    // - DSP-1: Several
    // - DSP-1A: Michael Andretti's Indy Car Challenge (also DSP-1), Ace o Nerae! 3D Tennis
    // - DSP-1B: Pilotwings (also DSP-1), Super Mario Kart (also DSP-1), Ballz 3D, Shutokō Battle 2: Drift King Keichii Tsuchiya & Masaaki Bandoh, Shutokō Battle '94: Keichii Tsuchiya Drift King
    Dsp1(Dsp1),    // 15+ games including Super Mario Kart and Pilotwings
    Dsp2,    // Dungeon Master only
    Dsp3,    // SD Gundam GX only
    Dsp4,    // Top Gear 3000 only
    // Versions of Super FX:
    // - Mario Chip: Star Fox (should be compatible with GSU-1?)
    // - GSU 1: Stunt Race FX, Vortex, Dirt Racer, Dirt Trax FX
    // - GSU 2: Super Mario World 2: Yoshi's Island, Doom, Winter Gold (PAL)
    SuperFx, // Many games including Star Fox, Super Mario World 2: Yoshi's Island, and Doom
    Sa1,     // 35 Super NES games, including Super Mario RPG: Legend of the Seven Stars
    Cx4,     // Mega Man X2 and Mega Man X3 only. "A Cx4 self-test screen can be accessed by holding the 'B' button on the second controller upon system start-up in both Mega Man X2 and X3"
    ObC1,   // Metal Combat: Falcon's Revenge only (super scope game)
    SDd1,   // Star Ocean and Street Fighter Alpha 2 only
    SRtc,    // Daikaijuu Monogatari II only
    Other(u8),
}

impl Coprocessor {
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Coprocessor::Dsp1(dsp::Dsp1::new()),
            1 => Coprocessor::SuperFx,
            2 => Coprocessor::ObC1,
            3 => Coprocessor::Sa1,
            4 => Coprocessor::SDd1,
            5 => Coprocessor::SRtc,
            x => Coprocessor::Other(x),
        }
    }

    pub fn is_implemented(&self) -> bool {
        match self {
            Coprocessor::Dsp1(_) => true,
            Coprocessor::Dsp2    => false,
            Coprocessor::Dsp3    => false,
            Coprocessor::Dsp4    => false,
            Coprocessor::SuperFx => false,
            Coprocessor::Sa1     => false,
            Coprocessor::Cx4     => false,
            Coprocessor::ObC1    => false,
            Coprocessor::SDd1    => false,
            Coprocessor::SRtc    => false,
            Coprocessor::Other(_) => false,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Coprocessor::Dsp1(_) => "DSP-1",
            Coprocessor::Dsp2    => "DSP-2",
            Coprocessor::Dsp3    => "DSP-3",
            Coprocessor::Dsp4    => "DSP-4",
            Coprocessor::SuperFx => "Super FX",
            Coprocessor::Sa1     => "SA-1",
            Coprocessor::Cx4     => "Cx4",
            Coprocessor::ObC1    => "OBC1",
            Coprocessor::SDd1    => "S-DD1",
            Coprocessor::SRtc    => "S-RTC",
            Coprocessor::Other(_) => "Other",
        }
    }
}