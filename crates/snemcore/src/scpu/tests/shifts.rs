//! shifts.rs — 65c816 CPU shift instruction verification suite.
//!
//! Coverage in this file (Tier 1 — complex instruction correctness):
//!   * ROL / ROR: accumulator and memory forms, 8-bit and 16-bit
//!   * ASL / LSR memory forms (accumulator forms already covered in chunk 1
//!     for ASL; LSR accumulator added here), 8-bit and 16-bit
//!
//! Opcodes used (from the provided dispatch table):
//!   ROL A=0x2A     ROL dp=0x26    ROR A=0x6A     ROR dp=0x66
//!   ASL dp=0x06    LSR A=0x4A     LSR dp=0x46
//!
//! Each test follows the format mandated by the steering doc:
//!   - Test name (with instruction + addressing mode)
//!   - Description
//!   - Input state
//!   - Expected output

use crate::scpu::*;
use super::common::*;

// ===========================================================================
// SECTION 6: ROL / ROR — Rotate Left / Right
// ===========================================================================

/// Test 65: ROL A — 8-bit, carry-in feeds bit 0, bit 7 feeds carry-out
/// Description: A=0x80 (bit 7 set), P.C=1 going in. Rotating left shifts
///              bit 7 into C (so C becomes 1, same value but from the old
///              bit 7) and the old carry-in (1) becomes the new bit 0.
///              0x80 rotated left with carry-in 1 -> 0x01, C=1.
/// Input: A=0xAB80, E=0, M=1, P.C=1,
///        MEM[00:8000]=0x2A (ROL A)
/// Expected Output: A=0xAB01, P.C=1, P.N=0, P.Z=0
#[test]
fn test_rol_accumulator_m8_carry_through() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x2A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0xAB80;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xAB01);
    assert!(cpu.is_flag_set(Flag::FlagC), "Old bit 7 (1) becomes new carry");
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 66: ROL dp — 8-bit memory, carry-in clear, bit 7 clear
/// Description: MEM=0x41 (0100_0001), C=0 going in. Rotating left:
///              0x41 << 1 = 0x82, with carry-in 0 filling bit 0, bit 7
///              (0) shifted out to C.
/// Input: E=0, M=1, DP=0x1000, P.C=0, MEM[00:1040]=0x41,
///        MEM[00:8000]=0x26 (ROL dp), MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0x82, P.C=0, P.N=1, P.Z=0
#[test]
fn test_rol_direct_page_m8() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x26, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x41]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x82);
    assert!(!cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 67: ROL A — 16-bit, bit 15 feeds carry-out
/// Description: A=0x8001, C=0 going in. Rotating left 16-bit: bit 15 (1)
///              shifts to C, carry-in (0) fills bit 0: 0x8001 -> 0x0002, C=1.
/// Input: A=0x8001, E=0, M=0, P.C=0,
///        MEM[00:8000]=0x2A
/// Expected Output: A=0x0002, P.C=1, P.N=0, P.Z=0
#[test]
fn test_rol_accumulator_m16() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x2A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x8001;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x0002);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 68: ROR A — 8-bit, carry-in feeds bit 7, bit 0 feeds carry-out
/// Description: A=0x01, C=1 going in. Rotating right: bit 0 (1) shifts to
///              C, carry-in (1) fills bit 7: 0x01 -> 0x80, C=1, N=1.
/// Input: A=0xCD01, E=0, M=1, P.C=1,
///        MEM[00:8000]=0x6A (ROR A)
/// Expected Output: A=0xCD80, P.C=1, P.N=1, P.Z=0
#[test]
fn test_ror_accumulator_m8_carry_through() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x6A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0xCD01;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xCD80);
    assert!(cpu.is_flag_set(Flag::FlagC), "Old bit 0 (1) becomes new carry");
    assert!(cpu.is_flag_set(Flag::FlagN), "Carry-in fills bit 7");
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 69: ROR dp — 8-bit memory, zero result
/// Description: MEM=0x01, C=0 going in. 0x01 >> 1 with carry-in 0 filling
///              bit 7 = 0x00, with the old bit 0 (1) becoming the new carry.
/// Input: E=0, M=1, DP=0x1000, P.C=0, MEM[00:1040]=0x01,
///        MEM[00:8000]=0x66 (ROR dp), MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0x00, P.C=1, P.Z=1, P.N=0
#[test]
fn test_ror_direct_page_m8_zero_result() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x66, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x00);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 70: ROR A — 16-bit, carry-in feeds bit 15
/// Description: A=0x0001, C=1 going in. Bit 0 (1) shifts to C; carry-in (1)
///              fills bit 15: 0x0001 -> 0x8000, C=1, N=1.
/// Input: A=0x0001, E=0, M=0, P.C=1,
///        MEM[00:8000]=0x6A
/// Expected Output: A=0x8000, P.C=1, P.N=1
#[test]
fn test_ror_accumulator_m16() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x6A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0001;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x8000);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagN));
}

/// Test 71: ROL A — Emulation mode (forces 8-bit rotate)
/// Description: In E=1, ROL A must rotate only the low 8 bits of A and
///              preserve the high byte (B accumulator), regardless of the
///              M flag's prior state.
/// Input: A=0xBEEF, E=1, P.C=0,
///        MEM[00:8000]=0x2A
/// Expected Output: A=0xBEDE, P.C=1 (old bit 7 of 0xEF is 1)
#[test]
fn test_rol_accumulator_emulation_mode() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x2A]);
    }
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0xBEEF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xBEDE, "0xEF (1110_1111) << 1 with carry-in 0 = 0xDE; high byte preserved");
    assert!(cpu.is_flag_set(Flag::FlagC), "Old bit 7 of 0xEF is 1");
}

// ===========================================================================
// SECTION 7: ASL / LSR — Arithmetic/Logical Shift (memory + LSR accumulator)
// ===========================================================================

/// Test 72: ASL dp — 8-bit memory, carry-out and zero result
/// Description: MEM=0x80. Shifting left: bit 7 (1) shifts to C, result is
///              0x00 (Z=1), bit 0 filled with 0 (no carry-in dependency for
///              ASL, unlike ROL).
/// Input: E=0, M=1, DP=0x1000, MEM[00:1040]=0x80,
///        MEM[00:8000]=0x06 (ASL dp), MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0x00, P.C=1, P.Z=1, P.N=0
#[test]
fn test_asl_direct_page_m8_carry_and_zero() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x06, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x80]);
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
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 73: ASL dp — 16-bit memory, bit 15 to carry
/// Description: MEM (16-bit) = 0x8001. Shift left: bit 15 (1) -> C,
///              result = 0x0002.
/// Input: E=0, M=0, DP=0x1000, MEM[00:1040..1042]=[0x01, 0x80],
///        MEM[00:8000]=0x06, MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040..1042]=[0x02, 0x00], P.C=1, P.N=0, P.Z=0
#[test]
fn test_asl_direct_page_m16() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x06, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x01, 0x80]);
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
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x02);
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1041)), 0x00);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 74: ASL dp — Emulation mode (forces 8-bit memory operand)
/// Description: In E=1, ASL on a direct-page operand must only read/write
///              a single byte regardless of prior M flag state.
/// Input: E=1, DP=0x1000, MEM[00:1040]=0x41,
///        MEM[00:8000]=0x06, MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040]=0x82, P.C=0, P.N=1
#[test]
fn test_asl_direct_page_emulation_mode() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x06, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x41]);
    }
    cpu.dp = 0x1000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x82);
    assert!(!cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagN));
}

/// Test 75: LSR A — 8-bit, bit 0 to carry, high bit always cleared
/// Description: A=0x01. Shifting right: bit 0 (1) -> C, bit 7 always filled
///              with 0 for LSR (unlike ROR), result = 0x00, Z=1, N=0.
/// Input: A=0xAB01, E=0, M=1,
///        MEM[00:8000]=0x4A (LSR A)
/// Expected Output: A=0xAB00, P.C=1, P.Z=1, P.N=0
#[test]
fn test_lsr_accumulator_m8_carry_and_zero() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x4A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0xAB01;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xAB00);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN), "LSR must always clear N (bit 7 forced to 0)");
}

/// Test 76: LSR dp — 16-bit memory
/// Description: MEM (16-bit) = 0x0003. Shift right: bit 0 (1) -> C,
///              result = 0x0001.
/// Input: E=0, M=0, DP=0x1000, MEM[00:1040..1042]=[0x03, 0x00],
///        MEM[00:8000]=0x46 (LSR dp), MEM[00:8001]=0x40
/// Expected Output: MEM[00:1040..1042]=[0x01, 0x00], P.C=1, P.Z=0, P.N=0
#[test]
fn test_lsr_direct_page_m16() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x46, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x03, 0x00]);
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
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1040)), 0x01);
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1041)), 0x00);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}