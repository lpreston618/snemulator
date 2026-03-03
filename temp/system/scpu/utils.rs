#![allow(dead_code)]

pub(super) trait CpuAddress {
    fn bank(self) -> u8;
    fn bank_addr(self) -> u16;
    fn page(self) -> u8;
    fn page_addr(self) -> u8;
    fn with_bank(self, bank: u8) -> Self;
    fn with_bank_addr(self, bank_addr: u16) -> Self;
    fn with_page(self, page: u8) -> Self;
    fn with_page_addr(self, page_addr: u8) -> Self;
    fn from_parts(bank: u8, page: u8, page_addr: u8) -> Self;

    fn wrapping_add8(self, value: u8) -> Self;
    fn wrapping_add16(self, value: u16) -> Self;
    fn wrapping_add24(self, value: u32) -> Self;
}

impl CpuAddress for u32 {
    fn bank(self) -> u8 {
        (self >> 16) as u8
    }
    fn bank_addr(self) -> u16 {
        self as u16
    }
    fn page(self) -> u8 {
        (self >> 8) as u8
    }
    fn page_addr(self) -> u8 {
        self as u8
    }
    fn with_bank(self, bank: u8) -> Self {
        ((bank as u32) << 16) | (self & 0x00FFFF)
    }
    fn with_bank_addr(self, bank_addr: u16) -> Self {
        (self & 0xFF0000) | (bank_addr as u32)
    }
    fn with_page(self, page: u8) -> Self {
        ((page as u32) << 8) | (self & 0xFF00FF)
    }
    fn with_page_addr(self, page_addr: u8) -> Self {
        (self & 0xFFFF00) | (page_addr as u32)
    }
    fn from_parts(bank: u8, page: u8, page_addr: u8) -> Self {
        ((bank as u32) << 16) | ((page as u32) << 8) | (page_addr as u32)
    }
    fn wrapping_add8(self, value: u8) -> Self {
        (self & 0xFFFF00) | ((self + value as u32) & 0xFF)
    }
    fn wrapping_add16(self, value: u16) -> Self {
        (self & 0xFF0000) | ((self + value as u32) & 0xFFFF)
    }
    fn wrapping_add24(self, value: u32) -> Self {
        (self + value) & 0xFFFFFF
    }
}

/// Returns a bool reporting whether an address lies in the memory region mapped 
/// to MMIO registers.
pub(super) fn is_mmio_addr(address: u32) -> bool {
    (address.bank() & 0x7F < 0x40) && (0x2000 <= address.bank_addr() && address.bank_addr() < 0x6000)
}

pub fn map_lorom_addr(address: u32) -> u32 {
    ((address & 0x7F0000) >> 1) | (address & 0x007FFF)
}

pub fn map_hirom_addr(address: u32) -> u32 {
    address & 0x3FFFFF
}

pub fn map_exhirom_addr(address: u32) -> u32 {
    (((address & 0x800000) ^ 0x800000) >> 1) | (address & 0x3FFFFF)
}