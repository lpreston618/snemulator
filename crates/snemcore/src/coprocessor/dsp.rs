// DSP-1 coprocessor emulation.
//
// Ported from bsnes's dsp1emu.cpp/.hpp (research by Overload, The Dumper,
// Neviksti and Andreas Naive), which is treated throughout as the primary
// reference in preference to the crazysmart.net.au DSP1/1A/1B writeup,
// since the latter contains several transcription bugs (see notes below).

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub const DSP_VERSION_1: usize = 0;
pub const DSP_VERSION_1A: usize = 0;
pub const DSP_VERSION_1B: usize = 1;

const RASTER_CMD: u8 = 0x0A;

// Status register bits (only the upper 8 bits of the 16-bit SR are exposed
// to the host CPU -- see Dsp1::getSr in the reference).
const SR_DRC: u8 = 0x04;
const SR_DRS: u8 = 0x10;
const SR_RQM: u8 = 0x80;

const MAX_READS: usize = 7;
const MAX_WRITES: usize = 1024;

const SINE_LUT: [i16; 256] = [
     0x0000,  0x0324,  0x0647,  0x096a,  0x0c8b,  0x0fab,  0x12c8,  0x15e2,
     0x18f8,  0x1c0b,  0x1f19,  0x2223,  0x2528,  0x2826,  0x2b1f,  0x2e11,
     0x30fb,  0x33de,  0x36ba,  0x398c,  0x3c56,  0x3f17,  0x41ce,  0x447a,
     0x471c,  0x49b4,  0x4c3f,  0x4ebf,  0x5133,  0x539b,  0x55f5,  0x5842,
     0x5a82,  0x5cb4,  0x5ed7,  0x60ec,  0x62f2,  0x64e8,  0x66cf,  0x68a6,
     0x6a6d,  0x6c24,  0x6dca,  0x6f5f,  0x70e2,  0x7255,  0x73b5,  0x7504,
     0x7641,  0x776c,  0x7884,  0x798a,  0x7a7d,  0x7b5d,  0x7c29,  0x7ce3,
     0x7d8a,  0x7e1d,  0x7e9d,  0x7f09,  0x7f62,  0x7fa7,  0x7fd8,  0x7ff6,
     0x7fff,  0x7ff6,  0x7fd8,  0x7fa7,  0x7f62,  0x7f09,  0x7e9d,  0x7e1d,
     0x7d8a,  0x7ce3,  0x7c29,  0x7b5d,  0x7a7d,  0x798a,  0x7884,  0x776c,
     0x7641,  0x7504,  0x73b5,  0x7255,  0x70e2,  0x6f5f,  0x6dca,  0x6c24,
     0x6a6d,  0x68a6,  0x66cf,  0x64e8,  0x62f2,  0x60ec,  0x5ed7,  0x5cb4,
     0x5a82,  0x5842,  0x55f5,  0x539b,  0x5133,  0x4ebf,  0x4c3f,  0x49b4,
     0x471c,  0x447a,  0x41ce,  0x3f17,  0x3c56,  0x398c,  0x36ba,  0x33de,
     0x30fb,  0x2e11,  0x2b1f,  0x2826,  0x2528,  0x2223,  0x1f19,  0x1c0b,
     0x18f8,  0x15e2,  0x12c8,  0x0fab,  0x0c8b,  0x096a,  0x0647,  0x0324,
    -0x0000, -0x0324, -0x0647, -0x096a, -0x0c8b, -0x0fab, -0x12c8, -0x15e2,
    -0x18f8, -0x1c0b, -0x1f19, -0x2223, -0x2528, -0x2826, -0x2b1f, -0x2e11,
    -0x30fb, -0x33de, -0x36ba, -0x398c, -0x3c56, -0x3f17, -0x41ce, -0x447a,
    -0x471c, -0x49b4, -0x4c3f, -0x4ebf, -0x5133, -0x539b, -0x55f5, -0x5842,
    -0x5a82, -0x5cb4, -0x5ed7, -0x60ec, -0x62f2, -0x64e8, -0x66cf, -0x68a6,
    -0x6a6d, -0x6c24, -0x6dca, -0x6f5f, -0x70e2, -0x7255, -0x73b5, -0x7504,
    -0x7641, -0x776c, -0x7884, -0x798a, -0x7a7d, -0x7b5d, -0x7c29, -0x7ce3,
    -0x7d8a, -0x7e1d, -0x7e9d, -0x7f09, -0x7f62, -0x7fa7, -0x7fd8, -0x7ff6,
    -0x7fff, -0x7ff6, -0x7fd8, -0x7fa7, -0x7f62, -0x7f09, -0x7e9d, -0x7e1d,
    -0x7d8a, -0x7ce3, -0x7c29, -0x7b5d, -0x7a7d, -0x798a, -0x7884, -0x776c,
    -0x7641, -0x7504, -0x73b5, -0x7255, -0x70e2, -0x6f5f, -0x6dca, -0x6c24,
    -0x6a6d, -0x68a6, -0x66cf, -0x64e8, -0x62f2, -0x60ec, -0x5ed7, -0x5cb4,
    -0x5a82, -0x5842, -0x55f5, -0x539b, -0x5133, -0x4ebf, -0x4c3f, -0x49b4,
    -0x471c, -0x447a, -0x41ce, -0x3f17, -0x3c56, -0x398c, -0x36ba, -0x33de,
    -0x30fb, -0x2e11, -0x2b1f, -0x2826, -0x2528, -0x2223, -0x1f19, -0x1c0b,
    -0x18f8, -0x15e2, -0x12c8, -0x0fab, -0x0c8b, -0x096a, -0x0647, -0x0324
];

const DATAROM: [i16; 1024] = [
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020,
    0x0040, 0x0080, 0x0100, 0x0200, 0x0400, 0x0800, 0x1000, 0x2000,
    0x4000, 0x7fff, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200,
    0x0100, 0x0080, 0x0040, 0x0020, 0x0001, 0x0008, 0x0004, 0x0002,
    0x0001, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, (0x8000u16 as i16), (0xffe5u16 as i16), 0x0100, 0x7fff, 0x7f02, 0x7e08,
    0x7d12, 0x7c1f, 0x7b30, 0x7a45, 0x795d, 0x7878, 0x7797, 0x76ba,
    0x75df, 0x7507, 0x7433, 0x7361, 0x7293, 0x71c7, 0x70fe, 0x7038,
    0x6f75, 0x6eb4, 0x6df6, 0x6d3a, 0x6c81, 0x6bca, 0x6b16, 0x6a64,
    0x69b4, 0x6907, 0x685b, 0x67b2, 0x670b, 0x6666, 0x65c4, 0x6523,
    0x6484, 0x63e7, 0x634c, 0x62b3, 0x621c, 0x6186, 0x60f2, 0x6060,
    0x5fd0, 0x5f41, 0x5eb5, 0x5e29, 0x5d9f, 0x5d17, 0x5c91, 0x5c0c,
    0x5b88, 0x5b06, 0x5a85, 0x5a06, 0x5988, 0x590b, 0x5890, 0x5816,
    0x579d, 0x5726, 0x56b0, 0x563b, 0x55c8, 0x5555, 0x54e4, 0x5474,
    0x5405, 0x5398, 0x532b, 0x52bf, 0x5255, 0x51ec, 0x5183, 0x511c,
    0x50b6, 0x5050, 0x4fec, 0x4f89, 0x4f26, 0x4ec5, 0x4e64, 0x4e05,
    0x4da6, 0x4d48, 0x4cec, 0x4c90, 0x4c34, 0x4bda, 0x4b81, 0x4b28,
    0x4ad0, 0x4a79, 0x4a23, 0x49cd, 0x4979, 0x4925, 0x48d1, 0x487f,
    0x482d, 0x47dc, 0x478c, 0x473c, 0x46ed, 0x469f, 0x4651, 0x4604,
    0x45b8, 0x456c, 0x4521, 0x44d7, 0x448d, 0x4444, 0x43fc, 0x43b4,
    0x436d, 0x4326, 0x42e0, 0x429a, 0x4255, 0x4211, 0x41cd, 0x4189,
    0x4146, 0x4104, 0x40c2, 0x4081, 0x4040, 0x3fff, 0x41f7, 0x43e1,
    0x45bd, 0x478d, 0x4951, 0x4b0b, 0x4cbb, 0x4e61, 0x4fff, 0x5194,
    0x5322, 0x54a9, 0x5628, 0x57a2, 0x5914, 0x5a81, 0x5be9, 0x5d4a,
    0x5ea7, 0x5fff, 0x6152, 0x62a0, 0x63ea, 0x6530, 0x6672, 0x67b0,
    0x68ea, 0x6a20, 0x6b53, 0x6c83, 0x6daf, 0x6ed9, 0x6fff, 0x7122,
    0x7242, 0x735f, 0x747a, 0x7592, 0x76a7, 0x77ba, 0x78cb, 0x79d9,
    0x7ae5, 0x7bee, 0x7cf5, 0x7dfa, 0x7efe, 0x7fff, 0x0020, 0x0040,
    0x0000, 0x0324, 0x0647, 0x096a, 0x0c8b, 0x0fab, 0x12c8, 0x15e2,
    0x18f8, 0x1c0b, 0x1f19, 0x2223, 0x2528, 0x2826, 0x2b1f, 0x2e11,
    0x30fb, 0x33de, 0x36ba, 0x398c, 0x3c56, 0x3f17, 0x41ce, 0x447a,
    0x471c, 0x49b4, 0x4c3f, 0x4ebf, 0x5133, 0x539b, 0x55f5, 0x5842,
    0x5a82, 0x5cb4, 0x5ed7, 0x60ec, 0x62f2, 0x64e8, 0x66cf, 0x68a6,
    0x6a6d, 0x6c24, 0x6dca, 0x6f5f, 0x70e2, 0x7255, 0x73b5, 0x7504,
    0x7641, 0x776c, 0x7884, 0x798a, 0x7a7d, 0x7b5d, 0x7c29, 0x7ce3,
    0x7d8a, 0x7e1d, 0x7e9d, 0x7f09, 0x7f62, 0x7fa7, 0x7fd8, 0x7ff6,
    0x7fff, 0x7ff6, 0x7fd8, 0x7fa7, 0x7f62, 0x7f09, 0x7e9d, 0x7e1d,
    0x7d8a, 0x7ce3, 0x7c29, 0x7b5d, 0x7a7d, 0x798a, 0x7884, 0x776c,
    0x7641, 0x7504, 0x73b5, 0x7255, 0x70e2, 0x6f5f, 0x6dca, 0x6c24,
    0x6a6d, 0x68a6, 0x66cf, 0x64e8, 0x62f2, 0x60ec, 0x5ed7, 0x5cb4,
    0x5a82, 0x5842, 0x55f5, 0x539b, 0x5133, 0x4ebf, 0x4c3f, 0x49b4,
    0x471c, 0x447a, 0x41ce, 0x3f17, 0x3c56, 0x398c, 0x36ba, 0x33de,
    0x30fb, 0x2e11, 0x2b1f, 0x2826, 0x2528, 0x2223, 0x1f19, 0x1c0b,
    0x18f8, 0x15e2, 0x12c8, 0x0fab, 0x0c8b, 0x096a, 0x0647, 0x0324,
    0x7fff, 0x7ff6, 0x7fd8, 0x7fa7, 0x7f62, 0x7f09, 0x7e9d, 0x7e1d,
    0x7d8a, 0x7ce3, 0x7c29, 0x7b5d, 0x7a7d, 0x798a, 0x7884, 0x776c,
    0x7641, 0x7504, 0x73b5, 0x7255, 0x70e2, 0x6f5f, 0x6dca, 0x6c24,
    0x6a6d, 0x68a6, 0x66cf, 0x64e8, 0x62f2, 0x60ec, 0x5ed7, 0x5cb4,
    0x5a82, 0x5842, 0x55f5, 0x539b, 0x5133, 0x4ebf, 0x4c3f, 0x49b4,
    0x471c, 0x447a, 0x41ce, 0x3f17, 0x3c56, 0x398c, 0x36ba, 0x33de,
    0x30fb, 0x2e11, 0x2b1f, 0x2826, 0x2528, 0x2223, 0x1f19, 0x1c0b,
    0x18f8, 0x15e2, 0x12c8, 0x0fab, 0x0c8b, 0x096a, 0x0647, 0x0324,
    0x0000, (0xfcdcu16 as i16), (0xf9b9u16 as i16), (0xf696u16 as i16), (0xf375u16 as i16), (0xf055u16 as i16), (0xed38u16 as i16), (0xea1eu16 as i16),
    (0xe708u16 as i16), (0xe3f5u16 as i16), (0xe0e7u16 as i16), (0xddddu16 as i16), (0xdad8u16 as i16), (0xd7dau16 as i16), (0xd4e1u16 as i16), (0xd1efu16 as i16),
    (0xcf05u16 as i16), (0xcc22u16 as i16), (0xc946u16 as i16), (0xc674u16 as i16), (0xc3aau16 as i16), (0xc0e9u16 as i16), (0xbe32u16 as i16), (0xbb86u16 as i16),
    (0xb8e4u16 as i16), (0xb64cu16 as i16), (0xb3c1u16 as i16), (0xb141u16 as i16), (0xaecdu16 as i16), (0xac65u16 as i16), (0xaa0bu16 as i16), (0xa7beu16 as i16),
    (0xa57eu16 as i16), (0xa34cu16 as i16), (0xa129u16 as i16), (0x9f14u16 as i16), (0x9d0eu16 as i16), (0x9b18u16 as i16), (0x9931u16 as i16), (0x975au16 as i16),
    (0x9593u16 as i16), (0x93dcu16 as i16), (0x9236u16 as i16), (0x90a1u16 as i16), (0x8f1eu16 as i16), (0x8dabu16 as i16), (0x8c4bu16 as i16), (0x8afcu16 as i16),
    (0x89bfu16 as i16), (0x8894u16 as i16), (0x877cu16 as i16), (0x8676u16 as i16), (0x8583u16 as i16), (0x84a3u16 as i16), (0x83d7u16 as i16), (0x831du16 as i16),
    (0x8276u16 as i16), (0x81e3u16 as i16), (0x8163u16 as i16), (0x80f7u16 as i16), (0x809eu16 as i16), (0x8059u16 as i16), (0x8028u16 as i16), (0x800au16 as i16),
    0x6488, 0x0080, 0x03ff, 0x0118, 0x0002, 0x0080, 0x4000, 0x3fd7,
    0x3faf, 0x3f86, 0x3f5d, 0x3f34, 0x3f0c, 0x3ee3, 0x3eba, 0x3e91,
    0x3e68, 0x3e40, 0x3e17, 0x3dee, 0x3dc5, 0x3d9c, 0x3d74, 0x3d4b,
    0x3d22, 0x3cf9, 0x3cd0, 0x3ca7, 0x3c7f, 0x3c56, 0x3c2d, 0x3c04,
    0x3bdb, 0x3bb2, 0x3b89, 0x3b60, 0x3b37, 0x3b0e, 0x3ae5, 0x3abc,
    0x3a93, 0x3a69, 0x3a40, 0x3a17, 0x39ee, 0x39c5, 0x399c, 0x3972,
    0x3949, 0x3920, 0x38f6, 0x38cd, 0x38a4, 0x387a, 0x3851, 0x3827,
    0x37fe, 0x37d4, 0x37aa, 0x3781, 0x3757, 0x372d, 0x3704, 0x36da,
    0x36b0, 0x3686, 0x365c, 0x3632, 0x3609, 0x35df, 0x35b4, 0x358a,
    0x3560, 0x3536, 0x350c, 0x34e1, 0x34b7, 0x348d, 0x3462, 0x3438,
    0x340d, 0x33e3, 0x33b8, 0x338d, 0x3363, 0x3338, 0x330d, 0x32e2,
    0x32b7, 0x328c, 0x3261, 0x3236, 0x320b, 0x31df, 0x31b4, 0x3188,
    0x315d, 0x3131, 0x3106, 0x30da, 0x30ae, 0x3083, 0x3057, 0x302b,
    0x2fff, 0x2fd2, 0x2fa6, 0x2f7a, 0x2f4d, 0x2f21, 0x2ef4, 0x2ec8,
    0x2e9b, 0x2e6e, 0x2e41, 0x2e14, 0x2de7, 0x2dba, 0x2d8d, 0x2d60,
    0x2d32, 0x2d05, 0x2cd7, 0x2ca9, 0x2c7b, 0x2c4d, 0x2c1f, 0x2bf1,
    0x2bc3, 0x2b94, 0x2b66, 0x2b37, 0x2b09, 0x2ada, 0x2aab, 0x2a7c,
    0x2a4c, 0x2a1d, 0x29ed, 0x29be, 0x298e, 0x295e, 0x292e, 0x28fe,
    0x28ce, 0x289d, 0x286d, 0x283c, 0x280b, 0x27da, 0x27a9, 0x2777,
    0x2746, 0x2714, 0x26e2, 0x26b0, 0x267e, 0x264c, 0x2619, 0x25e7,
    0x25b4, 0x2581, 0x254d, 0x251a, 0x24e6, 0x24b2, 0x247e, 0x244a,
    0x2415, 0x23e1, 0x23ac, 0x2376, 0x2341, 0x230b, 0x22d6, 0x229f,
    0x2269, 0x2232, 0x21fc, 0x21c4, 0x218d, 0x2155, 0x211d, 0x20e5,
    0x20ad, 0x2074, 0x203b, 0x2001, 0x1fc7, 0x1f8d, 0x1f53, 0x1f18,
    0x1edd, 0x1ea1, 0x1e66, 0x1e29, 0x1ded, 0x1db0, 0x1d72, 0x1d35,
    0x1cf6, 0x1cb8, 0x1c79, 0x1c39, 0x1bf9, 0x1bb8, 0x1b77, 0x1b36,
    0x1af4, 0x1ab1, 0x1a6e, 0x1a2a, 0x19e6, 0x19a1, 0x195c, 0x1915,
    0x18ce, 0x1887, 0x183f, 0x17f5, 0x17ac, 0x1761, 0x1715, 0x16c9,
    0x167c, 0x162e, 0x15df, 0x158e, 0x153d, 0x14eb, 0x1497, 0x1442,
    0x13ec, 0x1395, 0x133c, 0x12e2, 0x1286, 0x1228, 0x11c9, 0x1167,
    0x1104, 0x109e, 0x1036, 0x0fcc, 0x0f5f, 0x0eef, 0x0e7b, 0x0e04,
    0x0d89, 0x0d0a, 0x0c86, 0x0bfd, 0x0b6d, 0x0ad6, 0x0a36, 0x098d,
    0x08d7, 0x0811, 0x0736, 0x063e, 0x0519, 0x039a, 0x0000, 0x7fff,
    0x0100, 0x0080, 0x021f, 0x00c8, 0x00ce, 0x0048, 0x0a26, 0x277a,
    0x00ce, 0x6488, 0x14ac, 0x0001, 0x00f9, 0x00fc, 0x00ff, 0x00fc,
    0x00f9, (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16),
    (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16), (0xffffu16 as i16)
];
type Mat3 = [[i16; 3]; 3];

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
struct Vec3 { x: i16, y: i16, z: i16 }

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
struct DspSharedData {
    a: Mat3,          // attitude matrix A
    b: Mat3,
    c: Mat3,
    center: Vec3,     // center of projection
    center_zc: i16,
    center_ze: i16,
    voffset: i16,     // vertical offset of the screen w.r.t the center of projection
    les: i16,
    c_les: i16,
    e_les: i16,
    sin_aas: i16, cos_aas: i16,
    sin_azs: i16, cos_azs: i16,       // UNCLIPPED azimuth/zenith sin+cos (used by raster/target)
    sin_azs_b: i16, cos_azs_b: i16,   // CLIPPED zenith sin+cos (used only within `parameter`)
    secazs_c1: i16, secazs_e1: i16,
    secazs_c2: i16, secazs_e2: i16,
    norm: Vec3,
    global: Vec3,
    hx: i16, hy: i16,
    vertical: Vec3,
}

// ------------------------------------------------------------------------
// Fixed-point primitives
//
// These intentionally use i32 accumulation (dsp_mul, sum2, sum3) or
// wrapping ops (wrapping_neg) rather than plain i16 arithmetic, because the
// reference C++ relies on implicit int promotion before truncating back to
// int16 on assignment. Doing the arithmetic in plain i16 in Rust would
// panic on overflow in debug builds (and silently mis-truncate in release)
// wherever two fixed-point terms are summed, which happens constantly
// throughout these commands.
// ------------------------------------------------------------------------

/// (a * b) >> 15, i.e. Q1.15 fixed-point multiply.
const fn dsp_mul(a: i16, b: i16) -> i16 {
    (((a as i32) * (b as i32)) >> 15) as i16
}

/// (a * b) << 1. Reference's `N()` macro, used by normalize()/normalizeDouble().
const fn dsp_norm(a: i16, b: i16) -> i16 {
    (((a as i32) * (b as i32)) << 1) as i16
}

/// Sum of two i16 fixed-point values, widened through i32 then truncated
/// (mirrors C's int-promote-then-assign-to-int16 behavior).
const fn sum2(a: i16, b: i16) -> i16 {
    ((a as i32) + (b as i32)) as i16
}

const fn sum3(a: i16, b: i16, c: i16) -> i16 {
    ((a as i32) + (b as i32) + (c as i32)) as i16
}

/// DataRom[(base + offset)] where `offset` may legitimately be negative
/// (e.g. shift-table lookups with a negative exponent). Casting a negative
/// i16 straight to `usize` reinterprets its two's-complement bits as a huge
/// unsigned value rather than subtracting -- this widens to i32 first to
/// avoid that.
fn datarom_at(base: i16, offset: i16) -> i16 {
    DATAROM[(((base as i32) + (offset as i32)) as usize) & 0x3FF]
}

/// Counts the number of leading bits (below the sign bit) that match the
/// sign bit -- i.e. how far `c` can be left-shifted before its top two bits
/// differ. Mirrors the sign-dependent loop inlined twice in the reference's
/// `normalize`/`normalizeDouble` (branching on `m < 0`), which is *not* the
/// same as bit-complementing and running a single loop -- an earlier draft
/// tried that shortcut and got it wrong (see conversation history).
fn bsf(c: i16) -> i16 {
    let mut i: i16 = 0x4000;
    let mut e: i16 = 0;

    if c < 0 {
        while (c & i) != 0 && i != 0 {
            i >>= 1;
            e += 1;
        }
    } else {
        while (c & i) == 0 && i != 0 {
            i >>= 1;
            e += 1;
        }
    }

    e
}

fn dsp_sin(c: i16) -> i16 {
    if c < 0 {
        if c == i16::MIN { return 0; }
        return -dsp_sin(-c);
    }

    SINE_LUT[(c >> 8) as usize].saturating_add(dsp_mul(
        SINE_LUT[(0x40 + (c >> 8)) as usize],
        dsp_mul(0x6488, (c << 2) & 0x03FF),
    ))
}

fn dsp_cos(mut c: i16) -> i16 {
    if c < 0 {
        if c == i16::MIN { return i16::MIN; }
        c = -c;
    }

    SINE_LUT[(0x40 + (c >> 8)) as usize].saturating_sub(dsp_mul(
        SINE_LUT[(c >> 8) as usize],
        dsp_mul(0x6488, (c << 2) & 0x03FF),
    ))
}

/// Normalize `c` (treated as ranging -1..1) to Coefficient*2^Exponent with
/// |Coefficient| >= 1/2. `*exponent` is an accumulator: it is decremented by
/// the shift count, not overwritten.
fn dsp1_normalize(c: i16, coefficient: &mut i16, exponent: &mut i16) {
    let e = bsf(c);

    *coefficient = if e != 0 { dsp_norm(datarom_at(0x0021, e), c) } else { c };
    *exponent -= e;
}

/// Same as `dsp1_normalize` but takes a 32-bit product (e.g. the un-truncated
/// result of a multiply-accumulate) and writes an absolute (not accumulated)
/// exponent.
fn dsp1_normalize_double(product: i32, coefficient: &mut i16, exponent: &mut i16) {
    let n = (product & 0x7fff) as i16;
    let m = (product >> 15) as i16;
    let mut e = bsf(m);

    if e != 0 {
        *coefficient = dsp_norm(datarom_at(0x0021, e), m);

        if e < 15 {
            *coefficient += dsp_mul(datarom_at(0x0040, -e), n);
        } else {
            // Continue the same bit-scan onto `n`, using m's sign to pick
            // the direction -- see `bsf`'s doc comment for why this
            // "smuggle the sign bit in" trick is valid once `bsf` itself is
            // correct.
            e += bsf((m & i16::MIN) | n);

            if e > 15 {
                *coefficient = dsp_norm(datarom_at(0x0012, e), n);
            } else {
                *coefficient += n;
            }
        }
    } else {
        *coefficient = m;
    }

    *exponent = e;
}

/// Reference's `denormalizeAndClip` (the crazysmart PDF calls the same
/// operation `DSP1_Truncate` -- they are one function, not two; the earlier
/// draft of this file had both as separate, divergent copies).
fn dsp1_denormalize_and_clip(c: i16, e: i16) -> i16 {
    if e > 0 {
        if c > 0 { i16::MAX } else if c < 0 { -i16::MAX } else { c }
    } else if e < 0 {
        dsp_mul(c, datarom_at(0x0031, e))
    } else {
        c
    }
}

/// Reference's `shiftR`.
fn dsp1_shift_r(c: i16, e: i16) -> i16 {
    dsp_mul(c, datarom_at(0x0031, e))
}

// ------------------------------------------------------------------------
// Commands that don't touch shared state
// ------------------------------------------------------------------------

/// 00H - 16-bit Multiplication
fn dsp1_multiply(multiplicand: i16, multiplier: i16) -> i16 {
    dsp_mul(multiplicand, multiplier)
}

/// 20H - 16-bit Multiplication (alternate rounding)
fn dsp1_multiply1(multiplicand: i16, multiplier: i16) -> i16 {
    sum2(dsp_mul(multiplicand, multiplier), 1)
}

/// 10H - Inverse Calculation
fn dsp1_inverse(mut c: i16, mut e: i16, coefficient: &mut i16, exponent: &mut i16) {
    if c == 0 {
        *coefficient = i16::MAX;
        *exponent = 0x002F;
        return;
    }

    let mut sign: i16 = 1;

    if c < 0 {
        if c == i16::MIN { c = i16::MAX; } else { c = -c; }
        sign = -1;
    }

    while c < 0x4000 {
        c <<= 1;
        e -= 1;
    }

    if c == 0x4000 {
        if sign == 1 {
            *coefficient = i16::MAX;
        } else {
            *coefficient = -c;
            e -= 1;
        }
    } else {
        let mut i: i16 = DATAROM[(((c as usize) >> 7) & 0x7f) + 0x0065];

        // Two iterations of Newton's method on f(x) = 1/(2x) - c.
        i = sum2(i, dsp_mul(i.wrapping_neg(), dsp_mul(i, c))) << 1;
        i = sum2(i, dsp_mul(i.wrapping_neg(), dsp_mul(i, c))) << 1;

        *coefficient = i.wrapping_mul(sign);
    }

    *exponent = 1 - e;
}

/// 04H - Trigonometric Calculation
fn dsp1_triangle(theta: i16, radius: i16, s: &mut i16, c: &mut i16) {
    *s = dsp_mul(dsp_sin(theta), radius);
    *c = dsp_mul(dsp_cos(theta), radius);
}

/// 08H - Vector Size Calculation. Returns the raw (unsplit) 32-bit radius;
/// callers wire this into the output buffer as (low16, high16).
fn dsp1_radius(x: i16, y: i16, z: i16) -> i32 {
    let (x, y, z) = (x as i32, y as i32, z as i32);
    (x * x + y * y + z * z) << 1
}

/// 18H - Vector Size Comparison
fn dsp1_range(x: i16, y: i16, z: i16, radius: i16) -> i16 {
    let (x, y, z, radius) = (x as i32, y as i32, z as i32, radius as i32);
    ((x * x + y * y + z * z - radius * radius) >> 15) as i16
}

/// 38H - Vector Size Comparison (alternate rounding)
fn dsp1_range1(x: i16, y: i16, z: i16, radius: i16) -> i16 {
    sum2(dsp1_range(x, y, z, radius), 1)
}

/// 28H/38H shared table lookup - Vector Absolute Value Calculation.
///
/// Reference guards the `Pos & 1` rounding correction with
/// `#if DSP1_VERSION < 0x0102`, which covers *both* 0x0100 (DSP1/DSP1A) and
/// 0x0101 (DSP1B) -- i.e. it always applies for every documented chip
/// revision. An earlier draft mirrored the crazysmart.net.au PDF instead,
/// which guards it with `== 0x0100` and so silently skips the correction on
/// DSP1B. Going with bsnes (unconditional) here since it's the primary
/// reference and DSP1B titles are common in the supported game list.
fn dsp1_distance(x: i16, y: i16, z: i16) -> i16 {
    let (x, y, z) = (x as i32, y as i32, z as i32);
    let radius = x * x + y * y + z * z;

    if radius == 0 {
        return 0;
    }

    let (mut c, mut e) = (0i16, 0i16);
    dsp1_normalize_double(radius, &mut c, &mut e);
    if (e & 1) != 0 { c = dsp_mul(0x4000, c); }

    let pos = dsp_mul(0x0040, c) as i32;
    let node1 = DATAROM[(0x00D5 + pos) as usize] as i32;
    let node2 = DATAROM[(0x00D6 + pos) as usize] as i32;

    let mut distance = ((((node2 - node1) * ((c as i32) & 0x1FF)) >> 9) + node1) as i16;

    if (pos & 1) != 0 {
        distance = sum2(distance, -((node2 - node1) as i16));
    }

    distance >> (e >> 1)
}

/// 0CH - 2D Coordinate Rotation
fn dsp1_rotate(az: i16, x0: i16, y0: i16, xn: &mut i16, yn: &mut i16) {
    *xn = sum2(dsp_mul(dsp_sin(az), y0), dsp_mul(dsp_cos(az), x0));
    *yn = sum2(dsp_mul(dsp_cos(az), y0), -dsp_mul(dsp_sin(az), x0));
}

/// 1CH - 3D Coordinate Rotation
fn dsp1_polar(
    az: i16, ay: i16, ax: i16,
    x0: i16, y0: i16, z0: i16,
    xn: &mut i16, yn: &mut i16, zn: &mut i16,
) {
    let mut x1: i16 = 0;
    let mut y1: i16 = 0;
    let mut z1: i16 = 0;

    // Rotate Around Z
    dsp1_rotate(az, x0, y0, &mut x1, &mut y1);
    // Rotate Around Y
    dsp1_rotate(ay, z0, x1, &mut z1, xn);
    // Rotate Around X
    dsp1_rotate(ax, y1, z1, yn, zn);
}
// ------------------------------------------------------------------------
// Attitude-matrix commands (01H/11H/21H, 0DH/1DH/2DH, 03H/13H/23H,
// 0BH/1BH/2BH). The reference triplicates each of these three times (once
// per matrix A/B/C) with identical bodies; here they're written once and
// the three command wrappers (further down) just pick which matrix to
// read/write.
// ------------------------------------------------------------------------

/// 01H/11H/21H - Set Attitude A/B/C
fn dsp1_attitude(s: i16, rz: i16, ry: i16, rx: i16) -> Mat3 {
    let sin_rz = dsp_sin(rz);
    let cos_rz = dsp_cos(rz);
    let sin_ry = dsp_sin(ry);
    let cos_ry = dsp_cos(ry);
    let sin_rx = dsp_sin(rx);
    let cos_rx = dsp_cos(rx);

    let s = s >> 1;

    let m00 = dsp_mul(dsp_mul(s, cos_rz), cos_ry);
    let m01 = sum2(
        dsp_mul(dsp_mul(s, sin_rz), cos_rx),
        dsp_mul(dsp_mul(dsp_mul(s, cos_rz), sin_rx), sin_ry),
    );
    let m02 = sum2(
        dsp_mul(dsp_mul(s, sin_rz), sin_rx),
        -dsp_mul(dsp_mul(dsp_mul(s, cos_rz), cos_rx), sin_ry),
    );

    let m10 = -dsp_mul(dsp_mul(s, sin_rz), cos_ry);
    let m11 = sum2(
        dsp_mul(dsp_mul(s, cos_rz), cos_rx),
        -dsp_mul(dsp_mul(dsp_mul(s, sin_rz), sin_rx), sin_ry),
    );
    let m12 = sum2(
        dsp_mul(dsp_mul(s, cos_rz), sin_rx),
        dsp_mul(dsp_mul(dsp_mul(s, sin_rz), cos_rx), sin_ry),
    );

    let m20 = dsp_mul(s, sin_ry);
    let m21 = -dsp_mul(dsp_mul(s, sin_rx), cos_ry);
    let m22 = dsp_mul(dsp_mul(s, cos_rx), cos_ry);

    [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]]
}

/// 0DH/1DH/2DH - Convert from Global to Object Coordinates (F,L,U)
fn dsp1_objective(x: i16, y: i16, z: i16, m: &Mat3, f: &mut i16, l: &mut i16, u: &mut i16) {
    *f = sum3(dsp_mul(m[0][0], x), dsp_mul(m[1][0], y), dsp_mul(m[2][0], z));
    *l = sum3(dsp_mul(m[0][1], x), dsp_mul(m[1][1], y), dsp_mul(m[2][1], z));
    *u = sum3(dsp_mul(m[0][2], x), dsp_mul(m[1][2], y), dsp_mul(m[2][2], z));
}

/// 03H/13H/23H - Convert from Object to Global Coordinates (X,Y,Z)
fn dsp1_subjective(f: i16, l: i16, u: i16, m: &Mat3, x: &mut i16, y: &mut i16, z: &mut i16) {
    *x = sum3(dsp_mul(m[0][0], f), dsp_mul(m[0][1], l), dsp_mul(m[0][2], u));
    *y = sum3(dsp_mul(m[1][0], f), dsp_mul(m[1][1], l), dsp_mul(m[1][2], u));
    *z = sum3(dsp_mul(m[2][0], f), dsp_mul(m[2][1], l), dsp_mul(m[2][2], u));
}

/// 0BH/1BH/2BH - Inner product with the forward attitude column and a vector.
///
/// Note this is *not* three independent dsp_mul()s summed (unlike
/// `dsp1_objective`/`dsp1_subjective` above) -- the reference sums the raw
/// products first and only shifts once at the end, so precision/rounding
/// differs. Replicated faithfully.
fn dsp1_scalar(x: i16, y: i16, z: i16, m: &Mat3) -> i16 {
    (((x as i32) * (m[0][0] as i32)
        + (y as i32) * (m[1][0] as i32)
        + (z as i32) * (m[2][0] as i32))
        >> 15) as i16
}

/// 14H - 3D Angle Rotation
fn dsp1_gyrate(
    az: i16, ax: i16, ay: i16,
    u: i16, f: i16, l: i16,
    zn: &mut i16, xn: &mut i16, yn: &mut i16,
) {
    let sin_ay = dsp_sin(ay);
    let cos_ay = dsp_cos(ay);

    let (mut c_sec, mut e_sec) = (0i16, 0i16);
    dsp1_inverse(dsp_cos(ax), 0, &mut c_sec, &mut e_sec);

    // Rotation Around Z
    // NB: normalizeDouble takes the raw 32-bit product U*CosAy (int
    // promotion, no >>15), not dsp_mul(u, cos_ay) -- it's meant to receive
    // a full-width product it can then re-normalize.
    let (mut c, mut e) = (0i16, 0i16);
    dsp1_normalize_double(
        (u as i32) * (cos_ay as i32) - (f as i32) * (sin_ay as i32),
        &mut c,
        &mut e,
    );
    e = e_sec - e;
    dsp1_normalize(dsp_mul(c, c_sec), &mut c, &mut e);
    *zn = sum2(az, dsp1_denormalize_and_clip(c, e));

    // Rotation Around X
    *xn = sum3(ax, dsp_mul(u, sin_ay), dsp_mul(f, cos_ay));

    // Rotation Around Y
    dsp1_normalize_double(
        (u as i32) * (cos_ay as i32) + (f as i32) * (sin_ay as i32),
        &mut c,
        &mut e,
    );
    e = e_sec - e;
    let mut c_sin: i16 = 0;
    dsp1_normalize(dsp_sin(ax), &mut c_sin, &mut e);
    dsp1_normalize(-dsp_mul(c, dsp_mul(c_sec, c_sin)), &mut c, &mut e);
    *yn = sum3(ay, dsp1_denormalize_and_clip(c, e), l);
}
const MAXAZS_EXP: [i16; 16] = [
    0x38b4, 0x38b7, 0x38ba, 0x38be, 0x38c0, 0x38c4, 0x38c7, 0x38ca,
    0x38ce, 0x38d0, 0x38d4, 0x38d7, 0x38da, 0x38dd, 0x38e0, 0x38e4,
];

/// 02H - Projection Parameter Setting
///
/// Rewritten from scratch against the reference (the previous draft had a
/// typo'd variable name, a missing dereference on two outputs, and a
/// malformed dsp_mul() call, none of which would compile -- and it wrote
/// the clipped zenith sin/cos into the *unclipped* `sin_azs`/`cos_azs`
/// fields, which `raster`/`target` need later in their unclipped form).
fn dsp1_parameter(
    fx: i16, fy: i16, fz: i16, lfe: i16, les: i16, aas: i16, azs: i16,
    shared: &mut DspSharedData,
    vof: &mut i16, vva: &mut i16, cx: &mut i16, cy: &mut i16,
) {
    // Store Les and its coefficient/exponent when normalized.
    shared.les = les;
    shared.e_les = 0;
    dsp1_normalize(les, &mut shared.c_les, &mut shared.e_les);

    // Store sine/cosine of azimuth and (unclipped) zenith theta.
    shared.sin_aas = dsp_sin(aas);
    shared.cos_aas = dsp_cos(aas);
    shared.sin_azs = dsp_sin(azs);
    shared.cos_azs = dsp_cos(azs);

    // Normal vector to the screen (norm 1, points toward the center of projection).
    shared.norm.x = dsp_mul(shared.sin_azs, -shared.sin_aas);
    shared.norm.y = dsp_mul(shared.sin_azs, shared.cos_aas);
    shared.norm.z = dsp_mul(shared.cos_azs, 0x7FFF);

    // Horizontal vector of the screen (Hz=0, norm 1, points right).
    shared.hx = dsp_mul(shared.cos_aas, 0x7FFF);
    shared.hy = dsp_mul(shared.sin_aas, 0x7FFF);

    // Vertical vector of the screen (norm 1, points up).
    shared.vertical.x = dsp_mul(shared.cos_azs, -shared.sin_aas);
    shared.vertical.y = dsp_mul(shared.cos_azs, shared.cos_aas);
    shared.vertical.z = dsp_mul(-shared.sin_azs, 0x7FFF);

    let lfen = Vec3 {
        x: dsp_mul(lfe, shared.norm.x),
        y: dsp_mul(lfe, shared.norm.y),
        z: dsp_mul(lfe, shared.norm.z),
    };

    // Center of projection.
    shared.center.x = sum2(fx, lfen.x);
    shared.center.y = sum2(fy, lfen.y);
    shared.center.z = sum2(fz, lfen.z);

    let lesn = Vec3 {
        x: dsp_mul(les, shared.norm.x),
        y: dsp_mul(les, shared.norm.y),
        z: dsp_mul(les, shared.norm.z),
    };

    // Center of the screen (global coordinates).
    shared.global.x = sum2(shared.center.x, -lesn.x);
    shared.global.y = sum2(shared.center.y, -lesn.y);
    shared.global.z = sum2(shared.center.z, -lesn.z);

    let (mut c, mut e) = (0i16, 0i16);
    dsp1_normalize(shared.center.z, &mut c, &mut e);
    shared.center_zc = c;
    shared.center_ze = e;

    // Determine clip boundary and clip Zenith theta if necessary.
    let mut max_azs: i16 = MAXAZS_EXP[(-e) as usize];
    let mut azs_b = azs;
    if azs_b < 0 {
        max_azs = -max_azs;
        if azs_b < sum2(max_azs, 1) { azs_b = sum2(max_azs, 1); }
    } else if azs_b > max_azs {
        azs_b = max_azs;
    }

    // Store sine/cosine of the *clipped* zenith angle separately -- these
    // are only used locally below, they must not clobber shared.sin_azs /
    // shared.cos_azs (raster/target need the unclipped values).
    shared.sin_azs_b = dsp_sin(azs_b);
    shared.cos_azs_b = dsp_cos(azs_b);

    // Separation of (cx, cy) from the projection of the 'centre of
    // projection' over the ground (CentreZ * tan(AZS))...
    dsp1_inverse(shared.cos_azs_b, 0, &mut shared.secazs_c1, &mut shared.secazs_e1);
    dsp1_normalize(dsp_mul(c, shared.secazs_c1), &mut c, &mut e);
    e += shared.secazs_e1;
    c = dsp_mul(dsp1_denormalize_and_clip(c, e), shared.sin_azs_b);

    // ...then account for the centre of projection and the azimuth angle.
    shared.center.x = sum2(shared.center.x, dsp_mul(c, shared.sin_aas));
    shared.center.y = sum2(shared.center.y, -dsp_mul(c, shared.cos_aas));

    *cx = shared.center.x;
    *cy = shared.center.y;

    // Raster number of imaginary center and horizontal line.
    *vof = 0;
    let mut cos_azs_b = shared.cos_azs_b;

    if azs != azs_b || azs == max_azs {
        let azs_adj = if azs == i16::MIN { -i16::MAX } else { azs };

        let mut cc = azs_adj - max_azs;
        if cc >= 0 { cc -= 1; }
        let aux: i16 = !(cc << 2);

        cc = dsp_mul(datarom_at(0x0328, 0), aux);
        cc = sum2(dsp_mul(aux, cc), datarom_at(0x0327, 0));
        *vof -= dsp_mul(les, dsp_mul(cc, aux));

        let aux2 = dsp_mul(aux, aux);
        let aux3 = sum2(dsp_mul(datarom_at(0x0324, 0), aux2), datarom_at(0x0325, 0));
        cos_azs_b = sum2(cos_azs_b, dsp_mul(cos_azs_b, dsp_mul(aux2, aux3)));
    }

    shared.voffset = dsp_mul(les, cos_azs_b);

    let (mut c_sec, mut esec) = (0i16, 0i16);
    dsp1_inverse(shared.sin_azs_b, 0, &mut c_sec, &mut esec);
    dsp1_normalize(shared.voffset, &mut c, &mut esec);
    dsp1_normalize(dsp_mul(c_sec, c), &mut c, &mut esec);

    if c == i16::MIN {
        c >>= 1;
        esec += 1;
    }

    *vva = dsp1_denormalize_and_clip(-c, esec);

    // Store secant of the clipped zenith angle.
    dsp1_inverse(cos_azs_b, 0, &mut shared.secazs_c2, &mut shared.secazs_e2);
    shared.cos_azs_b = cos_azs_b;
}

/// 0AH - Raster Data Calculation
fn dsp1_raster(vs: i16, shared: &DspSharedData, an: &mut i16, bn: &mut i16, cn: &mut i16, dn: &mut i16) {
    let (mut c, mut e) = (0i16, 0i16);
    dsp1_inverse(sum2(dsp_mul(vs, shared.sin_azs), shared.voffset), 7, &mut c, &mut e);

    e += shared.center_ze;
    let c1 = dsp_mul(c, shared.center_zc);

    let mut e1 = e + shared.secazs_e2;

    dsp1_normalize(c1, &mut c, &mut e);
    c = dsp1_denormalize_and_clip(c, e);

    *an = dsp_mul(c, shared.cos_aas);
    *cn = dsp_mul(c, shared.sin_aas);

    dsp1_normalize(dsp_mul(c1, shared.secazs_c2), &mut c, &mut e1);
    c = dsp1_denormalize_and_clip(c, e1);

    *bn = dsp_mul(c, -shared.sin_aas);
    *dn = dsp_mul(c, shared.cos_aas);
}

/// 0EH - Coordinate Calculation of a Selected Point on the Screen
fn dsp1_target(h: i16, v: i16, shared: &DspSharedData, x: &mut i16, y: &mut i16) {
    let (mut c, mut e) = (0i16, 0i16);
    dsp1_inverse(sum2(dsp_mul(v, shared.sin_azs), shared.voffset), 8, &mut c, &mut e);

    e += shared.center_ze;
    let c1 = dsp_mul(c, shared.center_zc);
    let mut e1 = e + shared.secazs_e1;

    let h_scaled = h << 8;
    dsp1_normalize(c1, &mut c, &mut e);
    c = dsp_mul(dsp1_denormalize_and_clip(c, e), h_scaled);

    *x = sum2(shared.center.x, dsp_mul(c, shared.cos_aas));
    *y = sum2(shared.center.y, -dsp_mul(c, shared.sin_aas));

    let v_scaled = v << 8;
    dsp1_normalize(dsp_mul(c1, shared.secazs_c1), &mut c, &mut e1);
    c = dsp_mul(dsp1_denormalize_and_clip(c, e1), v_scaled);

    *x = sum2(*x, dsp_mul(c, -shared.sin_aas));
    *y = sum2(*y, dsp_mul(c, shared.cos_aas));
}

/// 06H - Object Projection Calculation
fn dsp1_project(x: i16, y: i16, z: i16, shared: &DspSharedData, h: &mut i16, v: &mut i16, m: &mut i16) {
    let (mut e, mut e3, mut e4) = (0i16, 0i16, 0i16);
    let (mut px, mut py, mut pz) = (0i16, 0i16, 0i16);

    dsp1_normalize_double((x as i32) - (shared.global.x as i32), &mut px, &mut e4);
    dsp1_normalize_double((y as i32) - (shared.global.y as i32), &mut py, &mut e);
    dsp1_normalize_double((z as i32) - (shared.global.z as i32), &mut pz, &mut e3);
    px >>= 1; e4 -= 1; // avoid overflow in the scalar products below
    py >>= 1; e -= 1;
    pz >>= 1; e3 -= 1;

    let mut ref_e = e.min(e3).min(e4);

    px = dsp1_shift_r(px, e4 - ref_e); // normalize to the same exponent
    py = dsp1_shift_r(py, e - ref_e);
    pz = dsp1_shift_r(pz, e3 - ref_e);

    let c11 = -dsp_mul(px, shared.norm.x);
    let c8 = -dsp_mul(py, shared.norm.y);
    let c9 = -dsp_mul(pz, shared.norm.z);
    let c12 = sum3(c11, c8, c9); // cannot overflow

    // De-normalize with 32-bit arithmetic.
    let mut aux4: i32 = c12 as i32;
    ref_e = 16 - ref_e; // can be up to 3
    if ref_e >= 0 {
        aux4 <<= ref_e;
    } else {
        aux4 >>= -ref_e;
    }
    if aux4 == -1 { aux4 = 0; } // reference keeps this odd-looking special case verbatim
    aux4 >>= 1;

    // Les - scalar product of P with the screen's normal vector.
    let aux: i32 = (shared.les as u16 as i32) + aux4;
    let (mut c10, mut e2) = (0i16, 0i16);
    dsp1_normalize_double(aux, &mut c10, &mut e2);
    e2 = 15 - e2;

    let (mut c4, mut e4b) = (0i16, 0i16);
    dsp1_inverse(c10, 0, &mut c4, &mut e4b);
    let c2 = dsp_mul(c4, shared.c_les); // scale factor

    // H
    let mut e7: i16 = 0;
    let c16 = dsp_mul(px, shared.hx);
    let c20 = dsp_mul(py, shared.hy);
    let c17 = sum2(c16, c20); // scalar product of P with the horizontal screen vector...
    let c18 = dsp_mul(c17, c2); // ...times the scale factor
    let mut c19: i16 = 0;
    dsp1_normalize(c18, &mut c19, &mut e7);
    *h = dsp1_denormalize_and_clip(c19, shared.e_les - e2 + ref_e + e7);

    // V
    let mut e6: i16 = 0;
    let c21 = dsp_mul(px, shared.vertical.x);
    let c22 = dsp_mul(py, shared.vertical.y);
    let c23 = dsp_mul(pz, shared.vertical.z);
    let c24 = sum3(c21, c22, c23); // scalar product of P with the vertical screen vector...
    let c26 = dsp_mul(c24, c2); // ...times the scale factor
    let mut c25: i16 = 0;
    dsp1_normalize(c26, &mut c25, &mut e6);
    *v = dsp1_denormalize_and_clip(c25, shared.e_les - e2 + ref_e + e6);

    // M (scale factor divided by 2^7)
    let mut c6: i16 = 0;
    dsp1_normalize(c2, &mut c6, &mut e4b);
    *m = dsp1_denormalize_and_clip(c6, e4b + shared.e_les - e2 - 7);
}
// ------------------------------------------------------------------------
// Memory commands
// ------------------------------------------------------------------------

/// 0FH - Memory Test
fn dsp1_memory_test() -> i16 {
    0x0000
}

/// 1FH - Transfer DATA ROM
///
/// NB: the reference literally does `memcpy(output, DataRom, 1024)` --
/// copying 1024 *bytes* (512 words) into a 1024-word output buffer, which
/// looks like an upstream off-by-half bug (the command table declares
/// `writes = 1024`, i.e. 1024 words are expected). This copies the full
/// 1024-word table instead. Flagging in case bug-for-bug fidelity is
/// wanted here instead -- this command is essentially never used by real
/// games, so it's a low-stakes call either way.
fn dsp1_memory_dump(out: &mut [i16; MAX_WRITES]) {
    out.copy_from_slice(&DATAROM);
}

/// 2FH - "Memory Size" in the reference (bsnes always returns the constant
/// 0x0100 here -- it is *not* the ROM-version query some docs label 0x2F
/// as). Following bsnes since it's authoritative.
fn dsp1_memory_size() -> i16 {
    0x0100
}

// ------------------------------------------------------------------------
// Command-table adapters
//
// The dispatch table needs one function-pointer type; these thin wrappers
// unpack the fixed-size read/write buffers into the named-argument core
// functions above (which stay directly testable/comparable against the
// reference in isolation).
// ------------------------------------------------------------------------

type CommandFn = fn(&[i16], &mut [i16], &mut DspSharedData);

struct Command {
    callback: Option<CommandFn>,
    reads: usize,
    writes: usize,
}

const NONE_CMD: Command = Command { callback: None, reads: 0, writes: 0 };

fn cmd_multiply(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_multiply(i[0], i[1]);
}
fn cmd_multiply1(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_multiply1(i[0], i[1]);
}
fn cmd_inverse(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let (o0, o1) = o.split_at_mut(1);
    dsp1_inverse(i[0], i[1], &mut o0[0], &mut o1[0]);
}
fn cmd_triangle(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let (mut s, mut c) = (0i16, 0i16);
    dsp1_triangle(i[0], i[1], &mut s, &mut c);
    o[0] = s;
    o[1] = c;
}
fn cmd_radius(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let r = dsp1_radius(i[0], i[1], i[2]);
    o[0] = r as i16;
    o[1] = (r >> 16) as i16;
}
fn cmd_range(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_range(i[0], i[1], i[2], i[3]);
}
fn cmd_range1(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_range1(i[0], i[1], i[2], i[3]);
}
fn cmd_distance(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_distance(i[0], i[1], i[2]);
}
fn cmd_rotate(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let (mut xn, mut yn) = (0i16, 0i16);
    dsp1_rotate(i[0], i[1], i[2], &mut xn, &mut yn);
    o[0] = xn;
    o[1] = yn;
}
fn cmd_polar(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let (mut xn, mut yn, mut zn) = (0i16, 0i16, 0i16);
    dsp1_polar(i[0], i[1], i[2], i[3], i[4], i[5], &mut xn, &mut yn, &mut zn);
    o[0] = xn;
    o[1] = yn;
    o[2] = zn;
}
fn cmd_parameter(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut vof, mut vva, mut cx, mut cy) = (0i16, 0i16, 0i16, 0i16);
    dsp1_parameter(i[0], i[1], i[2], i[3], i[4], i[5], i[6], s, &mut vof, &mut vva, &mut cx, &mut cy);
    o[0] = vof;
    o[1] = vva;
    o[2] = cx;
    o[3] = cy;
}
fn cmd_raster(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut an, mut bn, mut cn, mut dn) = (0i16, 0i16, 0i16, 0i16);
    dsp1_raster(i[0], s, &mut an, &mut bn, &mut cn, &mut dn);
    o[0] = an;
    o[1] = bn;
    o[2] = cn;
    o[3] = dn;
}
fn cmd_target(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut x, mut y) = (0i16, 0i16);
    dsp1_target(i[0], i[1], s, &mut x, &mut y);
    o[0] = x;
    o[1] = y;
}
fn cmd_project(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut h, mut v, mut m) = (0i16, 0i16, 0i16);
    dsp1_project(i[0], i[1], i[2], s, &mut h, &mut v, &mut m);
    o[0] = h;
    o[1] = v;
    o[2] = m;
}
fn cmd_gyrate(i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let (mut zn, mut xn, mut yn) = (0i16, 0i16, 0i16);
    dsp1_gyrate(i[0], i[1], i[2], i[3], i[4], i[5], &mut zn, &mut xn, &mut yn);
    o[0] = zn;
    o[1] = xn;
    o[2] = yn;
}
fn cmd_attitude_a(i: &[i16], _o: &mut [i16], s: &mut DspSharedData) {
    s.a = dsp1_attitude(i[0], i[1], i[2], i[3]);
}
fn cmd_attitude_b(i: &[i16], _o: &mut [i16], s: &mut DspSharedData) {
    s.b = dsp1_attitude(i[0], i[1], i[2], i[3]);
}
fn cmd_attitude_c(i: &[i16], _o: &mut [i16], s: &mut DspSharedData) {
    s.c = dsp1_attitude(i[0], i[1], i[2], i[3]);
}
fn cmd_objective_a(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut f, mut l, mut u) = (0i16, 0i16, 0i16);
    dsp1_objective(i[0], i[1], i[2], &s.a, &mut f, &mut l, &mut u);
    o[0] = f; o[1] = l; o[2] = u;
}
fn cmd_objective_b(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut f, mut l, mut u) = (0i16, 0i16, 0i16);
    dsp1_objective(i[0], i[1], i[2], &s.b, &mut f, &mut l, &mut u);
    o[0] = f; o[1] = l; o[2] = u;
}
fn cmd_objective_c(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut f, mut l, mut u) = (0i16, 0i16, 0i16);
    dsp1_objective(i[0], i[1], i[2], &s.c, &mut f, &mut l, &mut u);
    o[0] = f; o[1] = l; o[2] = u;
}
fn cmd_subjective_a(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut x, mut y, mut z) = (0i16, 0i16, 0i16);
    dsp1_subjective(i[0], i[1], i[2], &s.a, &mut x, &mut y, &mut z);
    o[0] = x; o[1] = y; o[2] = z;
}
fn cmd_subjective_b(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut x, mut y, mut z) = (0i16, 0i16, 0i16);
    dsp1_subjective(i[0], i[1], i[2], &s.b, &mut x, &mut y, &mut z);
    o[0] = x; o[1] = y; o[2] = z;
}
fn cmd_subjective_c(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    let (mut x, mut y, mut z) = (0i16, 0i16, 0i16);
    dsp1_subjective(i[0], i[1], i[2], &s.c, &mut x, &mut y, &mut z);
    o[0] = x; o[1] = y; o[2] = z;
}
fn cmd_scalar_a(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    o[0] = dsp1_scalar(i[0], i[1], i[2], &s.a);
}
fn cmd_scalar_b(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    o[0] = dsp1_scalar(i[0], i[1], i[2], &s.b);
}
fn cmd_scalar_c(i: &[i16], o: &mut [i16], s: &mut DspSharedData) {
    o[0] = dsp1_scalar(i[0], i[1], i[2], &s.c);
}
fn cmd_memory_test(_i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_memory_test();
}
fn cmd_memory_size(_i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    o[0] = dsp1_memory_size();
}
fn cmd_memory_dump(_i: &[i16], o: &mut [i16], _s: &mut DspSharedData) {
    let buf: &mut [i16; MAX_WRITES] = o.try_into().expect("memoryDump output must be MAX_WRITES long");
    dsp1_memory_dump(buf);
}

// ------------------------------------------------------------------------
// Command dispatch table -- mirrors Dsp1::mCommandTable exactly, including
// its duplicate slots (multiple opcodes alias the same command) and the
// three "freeze" slots (0x1a/0x2a/0x3a), which the FSM special-cases before
// ever consulting the table, so their entries are never actually invoked.
// ------------------------------------------------------------------------

const COMMAND_TABLE: [Command; 0x40] = [
    Command { callback: Some(cmd_multiply), reads: 2, writes: 1 },      // 0x00
    Command { callback: Some(cmd_attitude_a), reads: 4, writes: 0 },    // 0x01
    Command { callback: Some(cmd_parameter), reads: 7, writes: 4 },     // 0x02
    Command { callback: Some(cmd_subjective_a), reads: 3, writes: 3 },  // 0x03
    Command { callback: Some(cmd_triangle), reads: 2, writes: 2 },      // 0x04
    Command { callback: Some(cmd_attitude_a), reads: 4, writes: 0 },    // 0x05
    Command { callback: Some(cmd_project), reads: 3, writes: 3 },       // 0x06
    Command { callback: Some(cmd_memory_test), reads: 1, writes: 1 },   // 0x07
    Command { callback: Some(cmd_radius), reads: 3, writes: 2 },        // 0x08
    Command { callback: Some(cmd_objective_a), reads: 3, writes: 3 },   // 0x09
    Command { callback: Some(cmd_raster), reads: 1, writes: 4 },        // 0x0a (continuous mode)
    Command { callback: Some(cmd_scalar_a), reads: 3, writes: 1 },      // 0x0b
    Command { callback: Some(cmd_rotate), reads: 3, writes: 2 },        // 0x0c
    Command { callback: Some(cmd_objective_a), reads: 3, writes: 3 },   // 0x0d
    Command { callback: Some(cmd_target), reads: 2, writes: 2 },        // 0x0e
    Command { callback: Some(cmd_memory_test), reads: 1, writes: 1 },   // 0x0f

    Command { callback: Some(cmd_inverse), reads: 2, writes: 2 },       // 0x10
    Command { callback: Some(cmd_attitude_b), reads: 4, writes: 0 },    // 0x11
    Command { callback: Some(cmd_parameter), reads: 7, writes: 4 },     // 0x12
    Command { callback: Some(cmd_subjective_b), reads: 3, writes: 3 },  // 0x13
    Command { callback: Some(cmd_gyrate), reads: 6, writes: 3 },        // 0x14
    Command { callback: Some(cmd_attitude_b), reads: 4, writes: 0 },    // 0x15
    Command { callback: Some(cmd_project), reads: 3, writes: 3 },       // 0x16
    Command { callback: Some(cmd_memory_dump), reads: 1, writes: 1024 }, // 0x17
    Command { callback: Some(cmd_range), reads: 4, writes: 1 },         // 0x18
    Command { callback: Some(cmd_objective_b), reads: 3, writes: 3 },   // 0x19
    NONE_CMD,                                                          // 0x1a (freeze)
    Command { callback: Some(cmd_scalar_b), reads: 3, writes: 1 },      // 0x1b
    Command { callback: Some(cmd_polar), reads: 6, writes: 3 },         // 0x1c
    Command { callback: Some(cmd_objective_b), reads: 3, writes: 3 },   // 0x1d
    Command { callback: Some(cmd_target), reads: 2, writes: 2 },        // 0x1e
    Command { callback: Some(cmd_memory_dump), reads: 1, writes: 1024 }, // 0x1f

    Command { callback: Some(cmd_multiply1), reads: 2, writes: 1 },     // 0x20
    Command { callback: Some(cmd_attitude_c), reads: 4, writes: 0 },    // 0x21
    Command { callback: Some(cmd_parameter), reads: 7, writes: 4 },     // 0x22
    Command { callback: Some(cmd_subjective_c), reads: 3, writes: 3 },  // 0x23
    Command { callback: Some(cmd_triangle), reads: 2, writes: 2 },      // 0x24
    Command { callback: Some(cmd_attitude_c), reads: 4, writes: 0 },    // 0x25
    Command { callback: Some(cmd_project), reads: 3, writes: 3 },       // 0x26
    Command { callback: Some(cmd_memory_size), reads: 1, writes: 1 },   // 0x27
    Command { callback: Some(cmd_distance), reads: 3, writes: 1 },      // 0x28
    Command { callback: Some(cmd_objective_c), reads: 3, writes: 3 },   // 0x29
    NONE_CMD,                                                          // 0x2a (freeze)
    Command { callback: Some(cmd_scalar_c), reads: 3, writes: 1 },      // 0x2b
    Command { callback: Some(cmd_rotate), reads: 3, writes: 2 },        // 0x2c
    Command { callback: Some(cmd_objective_c), reads: 3, writes: 3 },   // 0x2d
    Command { callback: Some(cmd_target), reads: 2, writes: 2 },        // 0x2e
    Command { callback: Some(cmd_memory_size), reads: 1, writes: 1 },   // 0x2f

    Command { callback: Some(cmd_inverse), reads: 2, writes: 2 },       // 0x30
    Command { callback: Some(cmd_attitude_a), reads: 4, writes: 0 },    // 0x31
    Command { callback: Some(cmd_parameter), reads: 7, writes: 4 },     // 0x32
    Command { callback: Some(cmd_subjective_a), reads: 3, writes: 3 },  // 0x33
    Command { callback: Some(cmd_gyrate), reads: 6, writes: 3 },        // 0x34
    Command { callback: Some(cmd_attitude_a), reads: 4, writes: 0 },    // 0x35
    Command { callback: Some(cmd_project), reads: 3, writes: 3 },       // 0x36
    Command { callback: Some(cmd_memory_dump), reads: 1, writes: 1024 }, // 0x37
    Command { callback: Some(cmd_range1), reads: 4, writes: 1 },        // 0x38
    Command { callback: Some(cmd_objective_a), reads: 3, writes: 3 },   // 0x39
    NONE_CMD,                                                          // 0x3a (freeze)
    Command { callback: Some(cmd_scalar_a), reads: 3, writes: 1 },      // 0x3b
    Command { callback: Some(cmd_polar), reads: 6, writes: 3 },         // 0x3c
    Command { callback: Some(cmd_objective_a), reads: 3, writes: 3 },   // 0x3d
    Command { callback: Some(cmd_target), reads: 2, writes: 2 },        // 0x3e
    Command { callback: Some(cmd_memory_dump), reads: 1, writes: 1024 }, // 0x3f
];

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
enum FsmState {
    WaitCommand,
    ReadData,
    WriteData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dsp1 {
    sr: u8,
    sr_low_byte_access: bool,
    dr: u16,
    state: FsmState,
    command: u8,
    data_counter: usize,
    read_buffer: [i16; MAX_READS],
    #[serde(with = "BigArray")]
    write_buffer: [i16; MAX_WRITES],
    freeze: bool,
    shared: DspSharedData,
}

impl Dsp1 {
    pub fn new() -> Self {
        let mut dsp = Dsp1 {
            sr: 0,
            sr_low_byte_access: false,
            dr: 0,
            state: FsmState::WaitCommand,
            command: 0,
            data_counter: 0,
            read_buffer: [0; MAX_READS],
            write_buffer: [0; MAX_WRITES],
            freeze: false,
            shared: DspSharedData::default(),
        };
        dsp.reset();
        dsp
    }

    pub fn reset(&mut self) {
        self.sr = SR_DRC | SR_RQM;
        self.sr_low_byte_access = false;
        self.dr = 0x0080; // per reference: "only a supposition"
        self.freeze = false;
        self.state = FsmState::WaitCommand;
        self.shared = DspSharedData::default();
    }

    /// Only the upper 8 bits of the (conceptually 16-bit) status register
    /// are host-visible; each call toggles between returning 0 (low byte)
    /// and the real status (high byte).
    pub fn get_sr(&mut self) -> u8 {
        self.sr_low_byte_access = !self.sr_low_byte_access;
        if self.sr_low_byte_access { 0 } else { self.sr }
    }

    pub fn get_dr(&mut self) -> u8 {
        let mut data: u8 = 0;
        self.fsm_step(true, &mut data);
        data
    }

    pub fn set_dr(&mut self, value: u8) {
        let mut data = value;
        self.fsm_step(false, &mut data);
    }

    fn fsm_step(&mut self, read: bool, data: &mut u8) {
        if self.sr & SR_RQM == 0 {
            return;
        }

        // Bind the 8-bit bus access to the appropriate half of the 16-bit DR.
        if read {
            *data = if self.sr & SR_DRS != 0 {
                (self.dr >> 8) as u8
            } else {
                self.dr as u8
            };
        } else {
            if self.sr & SR_DRS != 0 {
                self.dr = (self.dr & 0x00ff) | ((*data as u16) << 8);
            } else {
                self.dr = (self.dr & 0xff00) | (*data as u16);
            }
        }

        match self.state {
            FsmState::WaitCommand => {
                self.command = self.dr as u8;
                if self.command & 0xc0 == 0 {
                    match self.command {
                        0x1a | 0x2a | 0x3a => {
                            self.freeze = true;
                        }
                        _ => {
                            self.data_counter = 0;
                            self.state = FsmState::ReadData;
                            self.sr &= !SR_DRC;
                        }
                    }
                }
            }
            FsmState::ReadData => {
                self.sr ^= SR_DRS;
                if self.sr & SR_DRS == 0 {
                    self.read_buffer[self.data_counter] = self.dr as i16;
                    self.data_counter += 1;

                    let cmd = &COMMAND_TABLE[self.command as usize];
                    if self.data_counter >= cmd.reads {
                        if let Some(callback) = cmd.callback {
                            callback(&self.read_buffer, &mut self.write_buffer, &mut self.shared);
                        }
                        if cmd.writes != 0 {
                            self.data_counter = 0;
                            self.dr = self.write_buffer[0] as u16;
                            self.state = FsmState::WriteData;
                        } else {
                            self.dr = 0x0080; // valid command completion
                            self.state = FsmState::WaitCommand;
                            self.sr |= SR_DRC;
                        }
                    }
                }
            }
            FsmState::WriteData => {
                self.sr ^= SR_DRS;
                if self.sr & SR_DRS == 0 {
                    self.data_counter += 1;
                    let cmd = &COMMAND_TABLE[self.command as usize];
                    if self.data_counter >= cmd.writes {
                        if self.command == RASTER_CMD && self.dr != 0x8000 {
                            // Continuous mode: advance to the next raster line.
                            self.read_buffer[0] = self.read_buffer[0].wrapping_add(1);
                            if let Some(callback) = cmd.callback {
                                callback(&self.read_buffer, &mut self.write_buffer, &mut self.shared);
                            }
                            self.data_counter = 0;
                            self.dr = self.write_buffer[0] as u16;
                        } else {
                            self.dr = 0x0080; // valid command completion
                            self.state = FsmState::WaitCommand;
                            self.sr |= SR_DRC;
                        }
                    } else {
                        self.dr = self.write_buffer[self.data_counter] as u16;
                    }
                }
            }
        }

        // RQM would be set here in every case except while frozen (0x1a/0x2a/0x3a).
        if self.freeze {
            self.sr &= !SR_RQM;
        }
    }
}

impl Default for Dsp1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    // A command byte is a single 8-bit write: WAIT_COMMAND never toggles
    // DRS, so it doesn't pair two accesses into one 16-bit word the way
    // READ_DATA/WRITE_DATA do.
    fn write_command(dsp: &mut Dsp1, cmd: u8) {
        dsp.set_dr(cmd);
    }

    // Parameter/result words *do* pair two 8-bit accesses (low byte first)
    // via the DRS toggle while in READ_DATA/WRITE_DATA state.
    fn write_word(dsp: &mut Dsp1, word: u16) {
        dsp.set_dr((word & 0xff) as u8);
        dsp.set_dr((word >> 8) as u8);
    }

    fn read_word(dsp: &mut Dsp1) -> u16 {
        let lo = dsp.get_dr() as u16;
        let hi = dsp.get_dr() as u16;
        lo | (hi << 8)
    }

    // Once the FSM returns to WAIT_COMMAND it stops pairing accesses, so
    // the 0x0080 "command complete" sentinel is read as a single byte.
    fn expect_complete(dsp: &mut Dsp1) {
        assert_eq!(dsp.get_dr(), 0x80);
    }

    #[test]
    fn multiply_roundtrip() {
        let mut dsp = Dsp1::new();
        write_command(&mut dsp, 0x00); // multiply
        write_word(&mut dsp, 0x4000); // 0.5
        write_word(&mut dsp, 0x4000); // 0.5
        let result = read_word(&mut dsp) as i16;
        assert_eq!(result, dsp_mul(0x4000, 0x4000));
        // command completion sentinel
        expect_complete(&mut dsp);
    }

    #[test]
    fn attitude_then_objective_roundtrip() {
        let mut dsp = Dsp1::new();
        // 0x01 = attitude A: S, Az, Ay, Ax -> no output words
        write_command(&mut dsp, 0x01); // attitude A
        write_word(&mut dsp, 0x7fff);
        write_word(&mut dsp, 0x0000);
        write_word(&mut dsp, 0x0000);
        write_word(&mut dsp, 0x0000);
        expect_complete(&mut dsp);

        // 0x0d = objective A: X, Y, Z -> F, L, U
        write_command(&mut dsp, 0x0d); // objective A
        write_word(&mut dsp, 0x1000);
        write_word(&mut dsp, 0x2000);
        write_word(&mut dsp, 0x3000);
        let _f = read_word(&mut dsp);
        let _l = read_word(&mut dsp);
        let _u = read_word(&mut dsp);
        expect_complete(&mut dsp);
    }

    #[test]
    fn parameter_then_raster_and_target_do_not_panic() {
        let mut dsp = Dsp1::new();
        // 0x02 = parameter: Fx,Fy,Fz,Lfe,Les,Aas,Azs -> Vof,Vva,Cx,Cy
        write_command(&mut dsp, 0x02); // parameter
        for v in [0x0000u16, 0x0000, 0x1000, 0x0100, 0x0200, 0x0000, 0x1000] {
            write_word(&mut dsp, v);
        }
        for _ in 0..4 { let _ = read_word(&mut dsp); }
        expect_complete(&mut dsp);

        // 0x0a = raster (continuous mode): Vs -> An,Bn,Cn,Dn.
        // Raster re-triggers itself for the next line unless the host
        // writes exactly 0x8000 in place of what would be the next read,
        // so do that to cleanly return to WAIT_COMMAND afterwards.
        write_command(&mut dsp, 0x0a); // raster
        write_word(&mut dsp, 0x0000);
        for _ in 0..4 { let _ = read_word(&mut dsp); }
        write_word(&mut dsp, 0x8000); // terminate continuous mode

        // 0x0e = target: H, V -> X, Y
        write_command(&mut dsp, 0x0e); // target
        write_word(&mut dsp, 0x0010);
        write_word(&mut dsp, 0x0020);
        let _x = read_word(&mut dsp);
        let _y = read_word(&mut dsp);
        expect_complete(&mut dsp);
    }

    #[test]
    fn project_with_negative_coordinates_does_not_panic() {
        let mut dsp = Dsp1::new();
        write_command(&mut dsp, 0x02); // parameter
        for v in [0x0000u16, 0x0000, 0x1000, 0x0100, 0x0200, 0x0000, 0x1000] {
            write_word(&mut dsp, v);
        }
        for _ in 0..4 { let _ = read_word(&mut dsp); }
        expect_complete(&mut dsp);

        // 0x06 = project: X, Y, Z (including negatives) -> H, V, M
        write_command(&mut dsp, 0x06); // project
        write_word(&mut dsp, (-500i16) as u16);
        write_word(&mut dsp, (-1000i16) as u16);
        write_word(&mut dsp, 2000u16);
        let _h = read_word(&mut dsp);
        let _v = read_word(&mut dsp);
        let _m = read_word(&mut dsp);
        expect_complete(&mut dsp);
    }

    #[test]
    fn memory_dump_fills_all_1024_words() {
        let mut dsp = Dsp1::new();
        write_command(&mut dsp, 0x1f); // memoryDump
        write_word(&mut dsp, 0x0000);
        let mut words = Vec::with_capacity(1024);
        for _ in 0..1024 {
            words.push(read_word(&mut dsp));
        }
        assert_eq!(words[0], 0x0000);
        expect_complete(&mut dsp);
    }

    #[test]
    fn bsf_matches_reference_semantics() {
        // e.g. -32767 has no leading redundant sign bits below the sign bit
        assert_eq!(bsf(-32767), 0);
        // -2 (0xFFFE) has 14
        assert_eq!(bsf(-2), 14);
        // -1 (0xFFFF) has 15
        assert_eq!(bsf(-1), 15);
        // 0 has 15 leading zero bits below the sign bit
        assert_eq!(bsf(0), 15);
    }
}