//! inc_dec.rs — 65c816 CPU INC/DEC instruction verification suite.
//!
//! Coverage in this file (Tier 1 — complex instruction correctness):
//!   * INC / DEC memory: wraparound edge cases, 8-bit and 16-bit
//!
//! Opcodes used:
//!   INC dp=0xE6    DEC dp=0xC6
//!
//! Each test follows the format mandated by the steering doc:
//!   - Test name (with instruction + addressing mode)
//!   - Description
//!   - Input state
//!   - Expected output

use crate::scpu::*;
use super::common::*;

// ===========================================================================
// SECTION 11: INC / DEC memory — Wraparound edge cases
// ===========================================================================

/// Test 91: INC dp — 8-bit memory, wraps 0xFF -> 0x00
/// Description: Incrementing 0xFF in 8-bit mode wraps to 0x00, setting Z=1.
///              INC/DEC do not affect the carry flag.
/// Input: E=0, M=1, DP=0x1000, MEM[00:1040]=0xFF,
///        MEM[00:8000]=0xE6 (INC dp), MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0x00, P.Z=1, P.N=0
#[test]
fn test_inc_direct_page_m8_wraps_to_zero() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0xFF]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x00);
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 92: INC dp — 16-bit memory, wraps 0xFFFF -> 0x0000
/// Description: Incrementing 0xFFFF in 16-bit mode wraps to 0x0000.
/// Input: E=0, M=0, DP=0x1000, MEM[00:1040..1042]=[0xFF, 0xFF],
///        MEM[00:8000]=0xE6, MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040..1042]=[0x00, 0x00], P.Z=1, P.N=0
#[test]
fn test_inc_direct_page_m16_wraps_to_zero() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0xFF, 0xFF]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x00);
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1041)), 0x00);
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 93: DEC dp — 8-bit memory, wraps 0x00 -> 0xFF
/// Description: Decrementing 0x00 in 8-bit mode wraps to 0xFF, setting N=1.
/// Input: E=0, M=1, DP=0x1000, MEM[00:1040]=0x00,
///        MEM[00:8000]=0xC6 (DEC dp), MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0xFF, P.N=1, P.Z=0
#[test]
fn test_dec_direct_page_m8_wraps_to_ff() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0xFF);
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 94: DEC dp — Emulation mode (forces 8-bit memory operand)
/// Description: In E=1, DEC on a direct-page operand wraps within a single
///              byte regardless of prior M flag state.
/// Input: E=1, DP=0x1000, MEM[00:1040]=0x00,
///        MEM[00:8000]=0xC6, MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0xFF, P.N=1, P.Z=0
#[test]
fn test_dec_direct_page_emulation_mode() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x00]);
    }
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0xFF);
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}