//! branches.rs — 65c816 CPU branching instruction verification suite.
//!
//! Coverage in this file:
//!
//! Opcodes used:
//!
//! Each test follows the format mandated by the steering doc:
//!   - Test name (with instruction + addressing mode)
//!   - Description
//!   - Input state
//!   - Expected output

use crate::scpu::*;
use super::common::*;

/// Test 39: BRA — Relative 8-bit addressing (forward)
/// Description: BRA always branches; offset is a signed 8-bit value added to
///              PC after the operand has been consumed (i.e., relative to
///              the byte after the branch instruction). Forward branch
///              with offset +0x10: PC after fetch is 0x8002, target is
///              0x8012. PB unchanged.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x80 (BRA), MEM[00:8001]=0x10
/// Expected Output: PB=0x00, PC=0x8012, branch_taken=true
#[test]
fn test_bra_relative_forward() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x80, 0x10]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0x8012, "BRA forward: 0x8002 + 0x10 = 0x8012");
    assert!(cpu.branch_taken, "BRA must set branch_taken");
}

/// Test 40: BRA — Relative 8-bit addressing (backward)
/// Description: Negative offset 0xFE (-2) makes BRA loop on itself: PC
///              after fetch is 0x8002, target is 0x8000.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x80 (BRA), MEM[00:8001]=0xFE
/// Expected Output: PB=0x00, PC=0x8000, branch_taken=true
#[test]
fn test_bra_relative_backward() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x80, 0xFE]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0x8000, "BRA backward: 0x8002 + (-2) = 0x8000");
    assert!(cpu.branch_taken);
}

/// Test 41: BRL — Relative long 16-bit addressing (forward)
/// Description: BRL takes a signed 16-bit offset added to PC after operand
///              consumption. Forward branch +0x1000: PC after fetch is
///              0x8003, target is 0x9003. PB unchanged.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x82 (BRL), MEM[00:8001]=0x00, MEM[00:8002]=0x10
/// Expected Output: PB=0x00, PC=0x9003, branch_taken=true
#[test]
fn test_brl_relative_long_forward() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x82, 0x00, 0x10]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0x9003, "BRL forward: 0x8003 + 0x1000 = 0x9003");
    assert!(cpu.branch_taken, "BRL must set branch_taken");
}

/// Test 42: BRL — Relative long 16-bit addressing (backward)
/// Description: Negative 16-bit offset 0xFFFD (-3) makes BRL loop on itself:
///              PC after fetch is 0x8003, target is 0x8000. Verifies sign
///              extension of the 16-bit offset.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x82 (BRL), MEM[00:8001]=0xFD, MEM[00:8002]=0xFF
/// Expected Output: PB=0x00, PC=0x8000, branch_taken=true
#[test]
fn test_brl_relative_long_backward() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x82, 0xFD, 0xFF]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0x8000, "BRL backward: 0x8003 + (-3) = 0x8000");
    assert!(cpu.branch_taken);
}

/// Test 43: BRL — Relative long 16-bit addressing (large forward)
/// Description: BRL with offset 0x7FFC (max positive) reaches near the end
///              of the bank. Verifies 16-bit offset arithmetic without
///              page-cross or bank-cross logic.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x82, MEM[00:8001]=0xFC, MEM[00:8002]=0x7F
/// Expected Output: PB=0x00, PC=0xFFFF, branch_taken=true
#[test]
fn test_brl_relative_long_max_forward() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x82, 0xFC, 0x7F]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0xFFFF, "BRL: 0x8003 + 0x7FFC = 0xFFFF");
    assert!(cpu.branch_taken);
}