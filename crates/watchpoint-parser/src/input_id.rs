use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputId {
    // CPU - Byte
    CpuPb,
    CpuDb,
    CpuP,
    CpuPrgByte0,
    CpuPrgByte1,
    CpuPrgByte2,
    CpuApuio0,
    CpuApuio1,
    CpuApuio2,
    CpuApuio3,
    CpuReg(CpuRegister),
    
    // CPU - Word
    CpuA,
    CpuX,
    CpuY,
    CpuSp,
    CpuPc,
    CpuDp,
    
    // CPU - Bool
    CpuFlagC,
    CpuFlagZ,
    CpuFlagI,
    CpuFlagD,
    CpuFlagX,
    CpuFlagM,
    CpuFlagV,
    CpuFlagN,
    CpuE,
    CpuHalted,
    CpuStopped,
    CpuNmiPending,
    CpuIrqPending,
    CpuWaiting,
    
    // CPU - Num
    CpuFullPc,
    
    // PPU - Byte
    PpuScreenBrightness,
    PpuObjSize,
    PpuBgMode,
    PpuMosaicSize,
    PpuCgramAddr,
    PpuWindow1Left,
    PpuWindow1Right,
    PpuWindow2Left,
    PpuWindow2Right,
    PpuReg(PpuRegister),
    
    // PPU - Word
    PpuNameBaseAddr,
    PpuNameSecondaryAddr,
    PpuOamAddr,
    PpuBg1TilemapAddr,
    PpuBg2TilemapAddr,
    PpuBg3TilemapAddr,
    PpuBg4TilemapAddr,
    PpuBg1Hofs,
    PpuBg1Vofs,
    PpuBg2Hofs,
    PpuBg2Vofs,
    PpuBg3Hofs,
    PpuBg3Vofs,
    PpuBg4Hofs,
    PpuBg4Vofs,
    PpuM7Hofs,
    PpuM7Vofs,
    PpuVramAddr,
    PpuM7A,
    PpuM7B,
    PpuM7C,
    PpuM7D,
    PpuM7X,
    PpuM7Y,
    PpuHCounter,
    PpuVCounter,
    
    // PPU - Bool
    PpuFBlank,
    PpuPriorityRotation,
    PpuBg1LargeTiles,
    PpuBg2LargeTiles,
    PpuBg3LargeTiles,
    PpuBg4LargeTiles,
    PpuBg3Mode1Priority,
    PpuBg1MosaicEnable,
    PpuBg2MosaicEnable,
    PpuBg3MosaicEnable,
    PpuBg4MosaicEnable,
    PpuBg1MainEnable,
    PpuBg2MainEnable,
    PpuBg3MainEnable,
    PpuBg4MainEnable,
    PpuObjMainEnable,
    
    // PPU - Num
    PpuDot,
    PpuScanline,
    PpuScreenX,
    PpuScreenY,
    PpuMultiplyResult,
    
    // APU - Byte
    ApuApuio0,
    ApuApuio1,
    ApuApuio2,
    ApuApuio3,
    
    // DMA - per channel (0-7)
    DmaAddrIncMode(u8),
    DmaTransferPattern(u8),
    DmaDmaSourceBank(u8),
    DmaHdmaIndirectTableBank(u8),
    DmaHdmaScanlineCounter(u8),
    DmaUnusedReg(u8),
    DmaBBusAddr(u8),
    DmaDmaSourceOffset(u8),
    DmaHdmaIndirectTableOffset(u8),
    DmaHdmaTableOffset(u8),
    DmaBToA(u8),
    DmaIndirectHdma(u8),
    DmaHdmaReload(u8),
    DmaFullDmaSourceAddr(u8),
    DmaFullHdmaIndirectTableAddr(u8),
    DmaReg(u8, DmaRegister),
    
    // System
    SysFrame,
    
    // Memory access (runtime-determined addresses)
    CpuMem,
    Wram,
    Vram,
    Aram,
    Oam,
    Cgram,
    Mmio,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CpuRegister {
    Apuio0, Apuio1, Apuio2, Apuio3,
    Wmdata, Wmaddl, Wmaddm, Wmaddh,
    Nmitimen, Wrmpya, Wrmpyb,
    Wrdivl, Wrdivh, Wrdivb,
    Htimel, Htimeh, Vtimel, Vtimeh,
    Mdmaen, Hdmaen, Memsel,
    Rdnmi, Timeup, Hvbjoy,
    Rddivl, Rddivh, Rdmpyl, Rdmpyh,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PpuRegister {
    Inidisp, Objsel, Oamaddl, Oamaddh,
    Bgmode, Mosaic, Bg1sc, Bg2sc, Bg3sc, Bg4sc,
    Bg12nba, Bg34nba,
    Bg1hofs, Bg1vofs, Bg2hofs, Bg2vofs,
    Bg3hofs, Bg3vofs, Bg4hofs, Bg4vofs,
    M7hofs, M7vofs,
    Vmain, Vmaddl, Vmaddh,
    M7sel, M7a, M7b, M7c, M7d, M7x, M7y,
    Cgadd,
    W12sel, W34sel, Wobjsel,
    Wh0, Wh1, Wh2, Wh3,
    Wbglog, Wobjlog,
    Tm, Ts, Tmw, Tsw,
    Cgwsel, Cgadsub, Coldata, Setini,
    Mpyl, Mpym, Mpyh,
    Slhv, Ophct, Opvct,
    Stat77, Stat78,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DmaRegister {
    Dmap, Bbad, A1tl, A1th, A1b,
    Dasl, Dash, Dasb, A2al, A2ah,
    Nltr, Unused,
}

impl fmt::Display for InputId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}