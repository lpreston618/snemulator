//! cpu_tests.rs — 65c816 CPU verification suite, chunk 2.
//!
//! Coverage in this file (Tier 1 — complex instruction correctness):
//!   * ADC: 8-bit, 16-bit, emulation-mode, binary carry/overflow edge cases,
//!          decimal-mode (BCD) cases
//!   * SBC: 8-bit, 16-bit, emulation-mode, binary borrow/overflow edge cases,
//!          decimal-mode (BCD) cases
//!   * CMP / CPX / CPY: equal/greater/less-than flag combinations
//!   * AND / ORA / EOR: flag-focused 8-bit and 16-bit cases
//!   * BIT: memory form (N/V/Z) and immediate form (Z only)
//!
//! Opcodes used:
//!   ADC #imm=0x69  SBC #imm=0xE9  CMP #imm=0xC9  CPX #imm=0xE0  CPY #imm=0xC0
//!   AND #imm=0x29  ORA #imm=0x09  EOR #imm=0x49
//!   BIT dp=0x24    BIT #imm=0x89
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
// SECTION 4: ADC — Add with Carry
// ===========================================================================

/// Test 44: ADC #imm — 8-bit, no carry-in, no overflow
/// Description: Basic 8-bit add with C=0 going in. 0x10 + 0x20 = 0x30,
///              no carry out, no signed overflow.
/// Input: A=0x0010, E=0, M=1, D=0, P.C=0,
///        MEM[00:8000]=0x69 (ADC #imm), MEM[00:8001]=0x20
/// Expected Output: A low=0x30, P.C=0, P.V=0, P.N=0, P.Z=0
#[test]
fn test_adc_immediate_m8_no_carry() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x20]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x0010;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0030);
    assert!(!cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagV));
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 45: ADC #imm — 8-bit, carry-in propagates into result
/// Description: With P.C=1 going in, 0x10 + 0x20 + 1 = 0x31.
/// Input: A=0x0010, E=0, M=1, D=0, P.C=1,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x20
/// Expected Output: A low=0x31, P.C=0
#[test]
fn test_adc_immediate_m8_carry_in() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x20]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0010;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0031);
    assert!(!cpu.is_flag_set(Flag::FlagC));
}

/// Test 46: ADC #imm — 8-bit, unsigned carry-out
/// Description: 0xFF + 0x02 = 0x101 -> wraps to 0x01 with C=1.
/// Input: A=0x00FF, E=0, M=1, D=0, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x02
/// Expected Output: A low=0x01, P.C=1, P.Z=0, P.N=0
#[test]
fn test_adc_immediate_m8_carry_out() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x02]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x00FF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0001);
    assert!(cpu.is_flag_set(Flag::FlagC), "0xFF + 0x02 must carry out");
    assert!(!cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 47: ADC #imm — 8-bit, signed overflow set (pos + pos = neg)
/// Description: 0x7F + 0x01 = 0x80. Result's sign bit flips despite both
///              operands being positive -> V=1. C=0 (no unsigned carry).
/// Input: A=0x007F, E=0, M=1, D=0, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x01
/// Expected Output: A low=0x80, P.V=1, P.N=1, P.C=0, P.Z=0
#[test]
fn test_adc_immediate_m8_signed_overflow() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x007F;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0080);
    assert!(cpu.is_flag_set(Flag::FlagV), "0x7F + 0x01 must set V (pos+pos=neg)");
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 48: ADC #imm — 8-bit, zero result
/// Description: 0xFF + 0x01 = 0x100 -> low byte 0x00, C=1, Z=1.
/// Input: A=0x00FF, E=0, M=1, D=0, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x01
/// Expected Output: A low=0x00, P.C=1, P.Z=1, P.N=0
#[test]
fn test_adc_immediate_m8_zero_result() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x00FF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0000);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 49: ADC #imm — 16-bit, carry-out at word boundary
/// Description: 0xFFFF + 0x0002 = 0x10001 -> wraps to 0x0001, C=1.
/// Input: A=0xFFFF, E=0, M=0, D=0, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x02, MEM[00:8002]=0x00
/// Expected Output: A=0x0001, P.C=1, P.Z=0, P.N=0
#[test]
fn test_adc_immediate_m16_carry_out() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x02, 0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0xFFFF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x0001);
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 50: ADC #imm — 16-bit, signed overflow set (pos + pos = neg)
/// Description: 0x7FFF + 0x0001 = 0x8000. V=1, N=1, C=0.
/// Input: A=0x7FFF, E=0, M=0, D=0, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x01, MEM[00:8002]=0x00
/// Expected Output: A=0x8000, P.V=1, P.N=1, P.C=0
#[test]
fn test_adc_immediate_m16_signed_overflow() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x01, 0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x7FFF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x8000);
    assert!(cpu.is_flag_set(Flag::FlagV));
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagC));
}

/// Test 51: ADC #imm — Emulation mode (forces 8-bit A regardless of M flag)
/// Description: In E=1, accumulator math is always 8-bit even if FlagM were
///              cleared beforehand; ADC must only consume one operand byte
///              and only affect A's low byte.
/// Input: A=0xAB10, E=1, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x05
/// Expected Output: A=0xAB15, PC=0x8002, P.C=0
#[test]
fn test_adc_immediate_emulation_mode() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x05]);
    }
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0xAB10;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xAB15, "Emulation mode ADC must preserve A high byte and only add to low byte");
    assert_eq!(cpu.pc, 0x8002, "Emulation mode ADC must consume exactly 1 operand byte");
    assert!(!cpu.is_flag_set(Flag::FlagC));
}

/// Test 52: ADC #imm — Decimal mode (BCD), 8-bit, no nibble correction needed
/// Description: D=1, 0x12 + 0x34 = 0x46 in BCD (no carry, no nibble fixup
///              required since 2+4=6 and 1+3=4, both <10).
/// Input: A=0x0012, E=0, M=1, D=1, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x34
/// Expected Output: A low=0x46, P.C=0
#[test]
fn test_adc_immediate_decimal_m8_no_fixup() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x34]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x0012;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0046, "BCD 0x12 + 0x34 = 0x46");
    assert!(!cpu.is_flag_set(Flag::FlagC));
}

/// Test 53: ADC #imm — Decimal mode (BCD), 8-bit, low-nibble carry fixup
/// Description: D=1, 0x19 + 0x01 = 0x20 in BCD. Binary add gives 0x1A; low
///              nibble (0xA) exceeds 9, so +6 correction carries into the
///              high nibble: 0x1A + 0x06 = 0x20.
/// Input: A=0x0019, E=0, M=1, D=1, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x01
/// Expected Output: A low=0x20, P.C=0
#[test]
fn test_adc_immediate_decimal_m8_low_nibble_fixup() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x0019;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0020, "BCD 0x19 + 0x01 = 0x20 (low-nibble fixup)");
    assert!(!cpu.is_flag_set(Flag::FlagC));
}

/// Test 54: ADC #imm — Decimal mode (BCD), 8-bit, decimal carry-out
/// Description: D=1, 0x99 + 0x01 = 0x100 in BCD -> wraps to 0x00 with C=1.
///              Per 65C816 (non-buggy) decimal behavior, Z reflects the
///              decimal result (0x00 -> Z=1).
/// Input: A=0x0099, E=0, M=1, D=1, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x01
/// Expected Output: A low=0x00, P.C=1, P.Z=1
#[test]
fn test_adc_immediate_decimal_m8_carry_out() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x0099;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0000, "BCD 0x99 + 0x01 = 0x00 with carry out");
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagZ));
}

/// Test 55: ADC #imm — Decimal mode (BCD), 16-bit, carry between bytes
/// Description: D=1, 16-bit BCD add: A=0x0099 + 0x0001 = 0x0100 in BCD
///              (low byte 99+01 decimal-carries to 00 and propagates a
///              carry into the high BCD byte: 00+00+1=01).
/// Input: A=0x0099, E=0, M=0, D=1, P.C=0,
///        MEM[00:8000]=0x69, MEM[00:8001]=0x01, MEM[00:8002]=0x00
/// Expected Output: A=0x0100, P.C=0 (carry consumed internally between BCD
///                  bytes, not out of the full 16-bit value)
#[test]
fn test_adc_immediate_decimal_m16_byte_carry() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x69, 0x01, 0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x0099;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x0100, "16-bit BCD 0099 + 0001 = 0100");
    assert!(!cpu.is_flag_set(Flag::FlagC), "No carry out of the full 16-bit BCD value");
}

// ===========================================================================
// SECTION 5: SBC — Subtract with Carry (borrow)
// ===========================================================================

/// Test 56: SBC #imm — 8-bit, no borrow-in (C=1 means no borrow per 6502/816 convention)
/// Description: With P.C=1 (no borrow), 0x30 - 0x10 = 0x20.
/// Input: A=0x0030, E=0, M=1, D=0, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x10
/// Expected Output: A low=0x20, P.C=1 (no borrow out), P.Z=0, P.N=0
#[test]
fn test_sbc_immediate_m8_no_borrow() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x10]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0030;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0020);
    assert!(cpu.is_flag_set(Flag::FlagC), "No borrow out -> C=1");
    assert!(!cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 57: SBC #imm — 8-bit, borrow-in (C=0 means borrow consumed)
/// Description: With P.C=0 going in, 0x30 - 0x10 - 1(borrow) = 0x1F.
/// Input: A=0x0030, E=0, M=1, D=0, P.C=0,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x10
/// Expected Output: A low=0x1F, P.C=1
#[test]
fn test_sbc_immediate_m8_borrow_in() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x10]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0x0030;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x001F);
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 58: SBC #imm — 8-bit, borrow-out (result goes negative/wraps)
/// Description: 0x10 - 0x20 (C=1, no incoming borrow) = -0x10 -> wraps to
///              0xF0 with C=0 (borrow out occurred).
/// Input: A=0x0010, E=0, M=1, D=0, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x20
/// Expected Output: A low=0xF0, P.C=0, P.N=1
#[test]
fn test_sbc_immediate_m8_borrow_out() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x20]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0010;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x00F0);
    assert!(!cpu.is_flag_set(Flag::FlagC), "Borrow out -> C=0");
    assert!(cpu.is_flag_set(Flag::FlagN));
}

/// Test 59: SBC #imm — 8-bit, signed overflow (pos - neg = neg, or
///           equivalently neg - pos overflow): 0x80 - 0x01 = 0x7F sets V
/// Description: 0x80 (-128) - 0x01 (1) = 0x7F. Crossing from negative to
///              positive via subtraction of a positive number sets V=1.
/// Input: A=0x0080, E=0, M=1, D=0, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x01
/// Expected Output: A low=0x7F, P.V=1, P.N=0, P.C=1
#[test]
fn test_sbc_immediate_m8_signed_overflow() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0080;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x007F);
    assert!(cpu.is_flag_set(Flag::FlagV), "0x80 - 0x01 must set V");
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 60: SBC #imm — 16-bit, borrow-out across word boundary
/// Description: 0x0000 - 0x0001 (C=1, no incoming borrow) wraps to 0xFFFF
///              with C=0.
/// Input: A=0x0000, E=0, M=0, D=0, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x01, MEM[00:8002]=0x00
/// Expected Output: A=0xFFFF, P.C=0, P.N=1
#[test]
fn test_sbc_immediate_m16_borrow_out() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x01, 0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.set_flag_to_bool(Flag::FlagD, false);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xFFFF);
    assert!(!cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagN));
}

/// Test 61: SBC #imm — Emulation mode (forces 8-bit A)
/// Description: In E=1, SBC only consumes one operand byte and only
///              touches A's low byte.
/// Input: A=0xCD30, E=1, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x10
/// Expected Output: A=0xCD20, PC=0x8002, P.C=1
#[test]
fn test_sbc_immediate_emulation_mode() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x10]);
    }
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0xCD30;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xCD20, "Emulation mode SBC must preserve A high byte");
    assert_eq!(cpu.pc, 0x8002);
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 62: SBC #imm — Decimal mode (BCD), 8-bit, no nibble correction
/// Description: D=1, C=1 (no borrow), 0x46 - 0x12 = 0x34 in BCD, no
///              correction needed since each nibble subtracts cleanly.
/// Input: A=0x0046, E=0, M=1, D=1, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x12
/// Expected Output: A low=0x34, P.C=1
#[test]
fn test_sbc_immediate_decimal_m8_no_fixup() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x12]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0046;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0034, "BCD 0x46 - 0x12 = 0x34");
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 63: SBC #imm — Decimal mode (BCD), 8-bit, low-nibble borrow fixup
/// Description: D=1, C=1 (no incoming borrow), 0x20 - 0x01 = 0x19 in BCD.
///              Binary subtraction underflows the low nibble (0-1), so a
///              -6 correction is applied after borrowing from the high
///              nibble: 0x20 - 0x01 = 0x1F binary, corrected to 0x19 BCD.
/// Input: A=0x0020, E=0, M=1, D=1, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x01
/// Expected Output: A low=0x19, P.C=1
#[test]
fn test_sbc_immediate_decimal_m8_low_nibble_fixup() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0020;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0019, "BCD 0x20 - 0x01 = 0x19 (low-nibble fixup)");
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 64: SBC #imm — Decimal mode (BCD), 8-bit, borrow-out
/// Description: D=1, C=1 (no incoming borrow), 0x00 - 0x01 = -1 in BCD,
///              wraps to 0x99 with C=0 (borrow out).
/// Input: A=0x0000, E=0, M=1, D=1, P.C=1,
///        MEM[00:8000]=0xE9, MEM[00:8001]=0x01
/// Expected Output: A low=0x99, P.C=0
#[test]
fn test_sbc_immediate_decimal_m8_borrow_out() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE9, 0x01]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagD, true);
    cpu.set_flag_to_bool(Flag::FlagC, true);
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0099, "BCD 0x00 - 0x01 = 0x99 with borrow out");
    assert!(!cpu.is_flag_set(Flag::FlagC));
}

// ===========================================================================
// SECTION 8: CMP / CPX / CPY — Compare
// ===========================================================================

/// Test 77: CMP #imm — 8-bit, operands equal
/// Description: A=0x40, operand=0x40. Equal values: Z=1, C=1 (no borrow,
///              since A >= operand), N=0.
/// Input: A=0x0040, E=0, M=1,
///        MEM[00:8000]=0xC9 (CMP #imm), MEM[00:8001]=0x40
/// Expected Output: A unchanged=0x0040, P.Z=1, P.C=1, P.N=0
#[test]
fn test_cmp_immediate_m8_equal() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC9, 0x40]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0x0040;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x0040, "CMP must not modify A");
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 78: CMP #imm — 8-bit, A greater than operand
/// Description: A=0x50, operand=0x10. A > operand: C=1, Z=0, N depends on
///              result sign (0x40 -> N=0).
/// Input: A=0x0050, E=0, M=1,
///        MEM[00:8000]=0xC9, MEM[00:8001]=0x10
/// Expected Output: P.C=1, P.Z=0, P.N=0
#[test]
fn test_cmp_immediate_m8_greater() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC9, 0x10]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0x0050;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 79: CMP #imm — 8-bit, A less than operand
/// Description: A=0x10, operand=0x50. A < operand: C=0 (borrow occurred),
///              Z=0, result 0x10-0x50=0xC0 -> N=1.
/// Input: A=0x0010, E=0, M=1,
///        MEM[00:8000]=0xC9, MEM[00:8001]=0x50
/// Expected Output: P.C=0, P.Z=0, P.N=1
#[test]
fn test_cmp_immediate_m8_less_than() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC9, 0x50]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0x0010;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert!(!cpu.is_flag_set(Flag::FlagC), "A < operand -> borrow -> C=0");
    assert!(!cpu.is_flag_set(Flag::FlagZ));
    assert!(cpu.is_flag_set(Flag::FlagN));
}

/// Test 80: CMP #imm — 16-bit, operands equal
/// Description: A=0x1234, operand=0x1234. Z=1, C=1.
/// Input: A=0x1234, E=0, M=0,
///        MEM[00:8000]=0xC9, MEM[00:8001]=0x34, MEM[00:8002]=0x12
/// Expected Output: P.Z=1, P.C=1, P.N=0
#[test]
fn test_cmp_immediate_m16_equal() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC9, 0x34, 0x12]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.a = 0x1234;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(cpu.is_flag_set(Flag::FlagC));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 81: CMP #imm — Emulation mode (forces 8-bit compare)
/// Description: In E=1, only A's low byte participates in the compare;
///              only one operand byte is consumed.
/// Input: A=0xFF40, E=1,
///        MEM[00:8000]=0xC9, MEM[00:8001]=0x40
/// Expected Output: PC=0x8002, P.Z=1, P.C=1
#[test]
fn test_cmp_immediate_emulation_mode() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC9, 0x40]);
    }
    cpu.a = 0xFF40;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pc, 0x8002);
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 82: CPX #imm — 8-bit (X-flag controlled, independent of M)
/// Description: Verifies CPX uses the X flag (not M) to determine operand
///              width. X=0x30, X-flag=1 (8-bit), operand=0x30 -> equal.
/// Input: X=0x0030, E=0, X-flag=1,
///        MEM[00:8000]=0xE0 (CPX #imm), MEM[00:8001]=0x30
/// Expected Output: PC=0x8002, P.Z=1, P.C=1
#[test]
fn test_cpx_immediate_x8_equal() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE0, 0x30]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, true);
    cpu.x = 0x0030;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pc, 0x8002, "8-bit CPX must consume exactly 1 operand byte");
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(cpu.is_flag_set(Flag::FlagC));
}

/// Test 83: CPY #imm — 16-bit, Y less than operand
/// Description: Y=0x0010, operand=0x0050, X-flag=0 (16-bit). Y < operand:
///              C=0, N=1 (result 0x10-0x50 wraps negative).
/// Input: Y=0x0010, E=0, X-flag=0,
///        MEM[00:8000]=0xC0 (CPY #imm), MEM[00:8001]=0x50, MEM[00:8002]=0x00
/// Expected Output: PC=0x8003, P.C=0, P.N=1, P.Z=0
#[test]
fn test_cpy_immediate_x16_less_than() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xC0, 0x50, 0x00]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.y = 0x0010;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pc, 0x8003, "16-bit CPY must consume exactly 2 operand bytes");
    assert!(!cpu.is_flag_set(Flag::FlagC));
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

// ===========================================================================
// SECTION 9: AND / ORA / EOR — Logical operations (flag-focused)
// ===========================================================================

/// Test 84: AND #imm — 8-bit, result zero
/// Description: A=0xF0 AND 0x0F = 0x00 -> Z=1, N=0.
/// Input: A=0x00F0, E=0, M=1,
///        MEM[00:8000]=0x29 (AND #imm), MEM[00:8001]=0x0F
/// Expected Output: A low=0x00, P.Z=1, P.N=0
#[test]
fn test_and_immediate_m8_zero_result() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x29, 0x0F]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0x00F0;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0000);
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

/// Test 85: AND #imm — 16-bit, high bit preserved sets N
/// Description: A=0xFF00 AND 0x8F00 = 0x8F00 -> N=1, Z=0.
/// Input: A=0xFF00, E=0, M=0,
///        MEM[00:8000]=0x29, MEM[00:8001]=0x00, MEM[00:8002]=0x8F
/// Expected Output: A=0x8F00, P.N=1, P.Z=0
#[test]
fn test_and_immediate_m16_negative_result() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x29, 0x00, 0x8F]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.a = 0xFF00;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x8F00);
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 86: ORA #imm — Emulation mode
/// Description: In E=1, ORA only affects A's low byte and consumes 1 byte.
/// Input: A=0x1200, E=1,
///        MEM[00:8000]=0x09 (ORA #imm), MEM[00:8001]=0x80
/// Expected Output: A=0x1280, PC=0x8002, P.N=1, P.Z=0
#[test]
fn test_ora_immediate_emulation_mode() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x09, 0x80]);
    }
    cpu.a = 0x1200;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x1280);
    assert_eq!(cpu.pc, 0x8002);
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 87: EOR #imm — 8-bit, self-XOR produces zero
/// Description: A=0x5A XOR 0x5A = 0x00 -> Z=1.
/// Input: A=0x005A, E=0, M=1,
///        MEM[00:8000]=0x49 (EOR #imm), MEM[00:8001]=0x5A
/// Expected Output: A low=0x00, P.Z=1, P.N=0
#[test]
fn test_eor_immediate_m8_self_xor_zero() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x49, 0x5A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0x005A;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0000);
    assert!(cpu.is_flag_set(Flag::FlagZ));
    assert!(!cpu.is_flag_set(Flag::FlagN));
}

// ===========================================================================
// SECTION 10: BIT — Test bits (memory form sets N/V/Z; immediate sets Z only)
// ===========================================================================

/// Test 88: BIT dp — Memory form sets N and V from operand, Z from AND
/// Description: Per 65C816 spec, the memory/non-immediate form of BIT
///              copies operand bit 7 to N and bit 6 to V directly (not
///              derived from the AND result), while Z is set from
///              (A AND operand) == 0. A=0x0F, MEM=0xC0 (1100_0000):
///              A AND MEM = 0x00 -> Z=1; N=1 (operand bit 7), V=1 (operand
///              bit 6). A itself is unmodified.
/// Input: A=0x000F, E=0, M=1, DP=0x1000, MEM[00:1040]=0xC0,
///        MEM[00:8000]=0x24 (BIT dp), MEM[00:8001]=0x40
/// Expected Output: A unchanged=0x000F, P.N=1, P.V=1, P.Z=1
#[test]
fn test_bit_direct_page_m8_n_v_from_operand() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x24, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0xC0]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.dp = 0x1000;
    cpu.a = 0x000F;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x000F, "BIT must not modify A");
    assert!(cpu.is_flag_set(Flag::FlagN), "N copied from operand bit 7");
    assert!(cpu.is_flag_set(Flag::FlagV), "V copied from operand bit 6");
    assert!(cpu.is_flag_set(Flag::FlagZ), "A AND operand == 0");
}

/// Test 89: BIT dp — Memory form, non-zero AND result clears Z
/// Description: A=0xFF, MEM=0x40 (0100_0000): A AND MEM = 0x40 != 0 -> Z=0.
///              N=0 (operand bit 7 clear), V=1 (operand bit 6 set).
/// Input: A=0x00FF, E=0, M=1, DP=0x1000, MEM[00:1040]=0x40,
///        MEM[00:8000]=0x24, MEM[00:8001]=0x40
/// Expected Output: P.N=0, P.V=1, P.Z=0
#[test]
fn test_bit_direct_page_m8_nonzero_and() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x24, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x40]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.dp = 0x1000;
    cpu.a = 0x00FF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(cpu.is_flag_set(Flag::FlagV));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 90: BIT #imm — Immediate form only affects Z (N/V untouched)
/// Description: Per 65C816 spec, the immediate-addressing form of BIT is
///              special-cased to only set Z; N and V are left at whatever
///              they were before the instruction (since there's no
///              "operand in memory" to copy bit 7/6 from in a meaningful
///              way). We pre-set N=1, V=1 and confirm they remain set even
///              though the AND result's high bits would otherwise suggest
///              clearing them.
/// Input: A=0x000F, E=0, M=1, P.N=1 (pre-set), P.V=1 (pre-set),
///        MEM[00:8000]=0x89 (BIT #imm), MEM[00:8001]=0xF0
/// Expected Output: A unchanged=0x000F, P.Z=1 (0x0F & 0xF0 == 0),
///                  P.N=1 (unchanged), P.V=1 (unchanged)
#[test]
fn test_bit_immediate_m8_only_z_affected() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x89, 0xF0]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagN, true);
    cpu.set_flag_to_bool(Flag::FlagV, true);
    cpu.a = 0x000F;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x000F);
    assert!(cpu.is_flag_set(Flag::FlagZ), "0x0F & 0xF0 == 0 -> Z=1");
    assert!(cpu.is_flag_set(Flag::FlagN), "Immediate BIT must not touch N");
    assert!(cpu.is_flag_set(Flag::FlagV), "Immediate BIT must not touch V");
}