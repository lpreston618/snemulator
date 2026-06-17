//! stack.rs — 65c816 CPU stack behavior verification suite.
//!
//! Coverage in this file:
//!   * Stack-modifying and reading instruction behavior
//!   * Edge cases in both native and emulation modes
//!
//! Each test follows the format mandated by the steering doc:
//!   - Test name (with instruction + addressing mode)
//!   - Description
//!   - Input state
//!   - Expected output

use crate::scpu::*;
use super::common::*;

// ===========================================================================
// SECTION 2: STACK HELPER BEHAVIOR (pin-down via observable side effects)
//
// These tests verify that push/pop wrap the SP within page 1 in emulation
// mode, while push_no_wrap/pop_no_wrap do not. We exercise this through
// observable instruction behavior rather than calling the helpers directly.
// ===========================================================================

/// Test 10: PHA — Stack wraps within page 1 in emulation mode
/// Description: With E=1 and SP=0x0100, pushing a byte must wrap SP to
///              0x01FF (8-bit stack page wrap).
/// Input: A=0x00AB, E=1, M=1, SP=0x0100, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x48 (PHA)
/// Expected Output: SP=0x01FF, MEM[00:0100]=0xAB
#[test]
fn test_pha_emulation_stack_wrap() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x48]);
    }
    cpu.a = 0x00AB;
    cpu.sp = 0x0100;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.sp, 0x01FF, "SP must wrap to 0x01FF after push at 0x0100");
    let mut bus = backing.bus();
    let pushed = cpu.read(&mut bus, addr(0x00, 0x0100));
    assert_eq!(pushed, 0xAB, "Byte should land at 0x0100 (the pre-decrement slot)");
}

/// Test 11: PLA — Stack wraps within page 1 in emulation mode
/// Description: With E=1 and SP=0x01FF, pulling a byte must wrap SP to 0x0100.
/// Input: E=1, M=1, SP=0x01FF, MEM[00:0100]=0x77, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x68 (PLA)
/// Expected Output: SP=0x0100, A low byte=0x77
#[test]
fn test_pla_emulation_stack_wrap() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x68]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x0100), &[0x77]);
    }
    cpu.sp = 0x01FF;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.sp, 0x0100, "SP must wrap to 0x0100 after pull at 0x01FF");
    assert_eq!(cpu.a & 0x00FF, 0x0077);
}

/// Test 12: PHA — Native mode push does not force page 1
/// Description: With E=0 and SP=0x1FFF, a push decrements SP to 0x1FFE
///              and writes the byte at 0x001FFF.
/// Input: A=0x00CD, E=0, M=1, SP=0x1FFF, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x48 (PHA)
/// Expected Output: SP=0x1FFE, MEM[00:1FFF]=0xCD
#[test]
fn test_pha_native_no_page1_forcing() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x48]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true); // 8-bit A
    cpu.a = 0x00CD;
    cpu.sp = 0x1FFF;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.sp, 0x1FFE);
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x1FFF)), 0xCD);
}

/// Test 13: PHD — Pins down whether PHD uses push_no_wrap in emulation mode
/// Description: PHD pushes the 16-bit DP register. Per the 65C816 reference,
///              PHD is one of the instructions that does NOT enforce page-1
///              stack wrap in emulation mode. With E=1 and SP=0x0100, PHD
///              should push DP high to 0x0100 and DP low to 0x00FF (no wrap),
///              wrapping only after the instruction, ending with SP=0x01FE.
/// Input: DP=0xBEEF, E=1, SP=0x0100, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x0B (PHD)
/// Expected Output: SP=0x00FE, MEM[00:0100]=0xBE, MEM[00:00FF]=0xEF
#[test]
fn test_phd_emulation_no_wrap() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x0B]);
    }
    cpu.dp = 0xBEEF;
    cpu.sp = 0x0100;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.sp, 0x01FE, "SP must be wrapped after instruction in emulation mode");
    let mut bus = backing.bus();
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x0100)), 0xBE);
    assert_eq!(cpu.read(&mut bus, addr(0x00, 0x00FF)), 0xEF, "PHD must not wrap SP during instruction in emulation mode");
}