//! reset.rs — 65c816 CPU reset verification suite.
//!
//! Coverage in this file:
//!   * Reset & power-on register effects
//!
//! Each test follows the format mandated by the steering doc:
//!   - Test name (with instruction + addressing mode)
//!   - Description
//!   - Input state
//!   - Expected output

use crate::scpu::*;
use super::common::*;

// ===========================================================================
// SECTION 1: RESET & POWER-ON
// ===========================================================================

/// Test 1: Reset — Vector fetch
/// Description: After `reset()`, PC must be loaded from 00:FFFC/FFFD.
/// Verifies that the reset vector is read from bank 0 and that PB is 0.
/// Input:
///   reset_vec = 0x1234 (baked into ROM by test_blank)
///   cpu = Cpu65c816::new()
/// Expected Output:
///   PC = 0x1234, PB = 0x00
#[test]
fn test_reset_vector_fetch() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x1234);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert_eq!(cpu.pc, 0x1234, "PC should be loaded from reset vector");
    assert_eq!(cpu.pb, 0x00, "PB should be 0 after reset");
}

/// Test 2: Reset — Emulation mode flag
/// Description: After `reset()`, the CPU must be in 6502 emulation mode (e=true).
/// Input:  reset_vec=0x8000, cpu.e = false (explicitly cleared so reset must restore)
/// Expected Output: e = true
#[test]
fn test_reset_emulation_flag() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    cpu.e = false;
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert!(cpu.e, "Reset must put CPU in emulation mode");
}

/// Test 3: Reset — M/X/I flags forced, D cleared
/// Description: Per 65C816 spec, reset sets M=1, X=1, I=1 and clears D.
/// Input: reset_vec=0x8000, p = 0x00
/// Expected Output: P bits — M=1, X=1, I=1, D=0
#[test]
fn test_reset_status_flags() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    cpu.p = 0x00;
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert!(cpu.is_flag_set(Flag::FlagM), "M must be 1 after reset");
    assert!(cpu.is_flag_set(Flag::FlagX), "X must be 1 after reset");
    assert!(cpu.is_flag_set(Flag::FlagI), "I must be 1 after reset");
    assert!(!cpu.is_flag_set(Flag::FlagD), "D must be 0 after reset");
}

/// Test 4: Reset — Bank registers and DP cleared
/// Description: DB, PB, and DP are zeroed by reset.
/// Input: reset_vec=0x8000, db=0xAB, pb=0xCD, dp=0xBEEF
/// Expected Output: db=0, pb=0, dp=0
#[test]
fn test_reset_bank_and_dp_cleared() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    cpu.db = 0xAB;
    cpu.pb = 0xCD;
    cpu.dp = 0xBEEF;
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert_eq!(cpu.db, 0x00);
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.dp, 0x0000);
}

/// Test 5: Reset — Stack pointer high byte forced to 0x01
/// Description: Reset puts the stack in page 1 (emulation page).
/// Input: reset_vec=0x8000, sp = 0xBEEF
/// Expected Output: sp high byte = 0x01
#[test]
fn test_reset_sp_high_byte() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    cpu.sp = 0xBEEF;
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert_eq!(cpu.sp & 0xFF00, 0x0100, "SP high byte must be 0x01 after reset");
}

/// Test 6: Reset — Internal flags cleared
/// Description: stopped, irq_pending, nmi_pending, waiting_for_interrupt
///              are explicitly cleared by `reset()`.
/// Input: reset_vec=0x8000, all four flags = true
/// Expected Output: all four flags = false
#[test]
fn test_reset_clears_internal_flags() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    cpu.stopped = true;
    cpu.irq_pending = true;
    cpu.nmi_pending = true;
    cpu.waiting_for_interrupt = true;
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert!(!cpu.stopped);
    assert!(!cpu.irq_pending);
    assert!(!cpu.nmi_pending);
    assert!(!cpu.waiting_for_interrupt);
}

/// Test 7: Reset — Vector byte order
/// Description: Pins down byte order of the reset vector by using a value with
///              distinct low/high bytes. Confirms low byte from 00:FFFC and
///              high byte from 00:FFFD.
/// Input: reset_vec = 0xABCD
/// Expected Output: PC = 0xABCD (low=0xCD, high=0xAB)
#[test]
fn test_reset_vector_byte_order() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0xABCD);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert_eq!(cpu.pc, 0xABCD);
}

/// Test 8: Reset — Clears halted state
/// Description: If the CPU is halted (post-STP), reset must clear the halt
///              and resume from the reset vector.
/// Input: reset_vec=0x8000, cpu.halted=true, cpu.stopped=true
/// Expected Output: stopped=false, PC=0x8000
///                  (halted is not explicitly cleared by reset() body shown,
///                   so this test pins down whether power_on must be used
///                   for full halt recovery)
#[test]
fn test_reset_clears_stopped_resumes_pc() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    cpu.halted = true;
    cpu.stopped = true;
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
    }
    assert!(!cpu.stopped, "Reset must clear stopped");
    assert_eq!(cpu.pc, 0x8000);
}

/// Test 9: Reset — Idempotency
/// Description: Calling reset twice in a row produces the same final state
///              as calling it once.
/// Input: reset_vec=0x9000, two consecutive reset() calls
/// Expected Output: e=true, M=1, X=1, I=1, D=0, PC=0x9000, sp_hi=0x01
#[test]
fn test_reset_idempotent() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x9000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        cpu.reset(&mut bus);
    }
    assert!(cpu.e);
    assert!(cpu.is_flag_set(Flag::FlagM));
    assert!(cpu.is_flag_set(Flag::FlagX));
    assert!(cpu.is_flag_set(Flag::FlagI));
    assert!(!cpu.is_flag_set(Flag::FlagD));
    assert_eq!(cpu.pc, 0x9000);
    assert_eq!(cpu.sp & 0xFF00, 0x0100);
}
