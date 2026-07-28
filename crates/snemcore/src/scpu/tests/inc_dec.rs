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

#![allow(unused_imports)]

use crate::{debug::NullHarness, scpu::*};
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
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0xFF]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    // INC dp (8-bit, m=1) is 5 cycles (7-2m+w, w=0). 4 cycles move a byte
    // over the bus (opcode fetch, operand fetch, RMW read, RMW write); on
    // the 65816 the dummy RMW write of the old value that the NMOS 6502
    // performs becomes a pure internal cycle instead, so 1 cycle is internal:
    // 4 * 8 + 1 * 6 = 38 master clocks.
    // Checked here, before the verification read below, since that read is
    // itself an instrumented bus access that would add its own clocks.
    assert_eq!(cpu.clocks, 38, "INC dp (8-bit, DL=0) must take 38 master clocks (4 bus bytes + 1 internal cycle)");
    let mut bus = backing.bus(&mut harness);
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
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0xFF, 0xFF]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    // INC dp (16-bit, m=0) is 7 cycles (7-0+w, w=0). 6 cycles move a byte
    // over the bus (opcode fetch, operand fetch, low+high RMW reads,
    // low+high RMW writes), leaving 1 internal cycle:
    // 6 * 8 + 1 * 6 = 54 master clocks.
    // Checked before the verification reads below, which would add their own clocks.
    assert_eq!(cpu.clocks, 54, "INC dp (16-bit, DL=0) must take 54 master clocks (6 bus bytes + 1 internal cycle)");
    let mut bus = backing.bus(&mut harness);
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
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    // DEC dp uses the same cycle formula as INC dp: 5 cycles, 4 bus bytes
    // (opcode, operand, RMW read, RMW write), 1 internal cycle.
    // 4 * 8 + 1 * 6 = 38 master clocks. Checked before the verification read below.
    assert_eq!(cpu.clocks, 38, "DEC dp (8-bit, DL=0) must take 38 master clocks (4 bus bytes + 1 internal cycle)");
    let mut bus = backing.bus(&mut harness);
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
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC6, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x00]);
    }
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    // Emulation mode forces m=1 (per reset()), same as the native 8-bit case:
    // 4 bus bytes + 1 internal cycle = 4 * 8 + 1 * 6 = 38 master clocks.
    // Checked before the verification read below.
    assert_eq!(cpu.clocks, 38, "DEC dp (emulation mode, DL=0) must take 38 master clocks (4 bus bytes + 1 internal cycle)");
    let mut bus = backing.bus(&mut harness);
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0xFF);
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}