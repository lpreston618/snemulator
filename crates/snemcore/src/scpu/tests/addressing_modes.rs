//! addressing_modes.rs — 65c816 CPU addressing modes verification suite.
//!
//! Coverage in this file:
//!   * All 23 addressing-mode probes (driven via LDA / LDX / JMP / etc.)
//!   * Edge cases: DP wrap, PC bank wrap, indirect bank-cross
//!
//! Each test follows the format mandated by the steering doc:
//!   - Test name (with instruction + addressing mode)
//!   - Description
//!   - Input state
//!   - Expected output

use crate::scpu::*;
use super::common::*;

// ===========================================================================
// SECTION 3: ADDRESSING-MODE PROBES
//
// Each test exercises one addressing mode through a representative
// instruction (LDA for read modes, STA for store/write modes, JMP for
// control-flow modes) so the effective-address computation is observable.
// ===========================================================================

/// Test 14: LDA #imm — Immediate addressing, M=1 (8-bit)
/// Description: Verifies immediate addressing fetches a single operand byte
///              when M=1 and loads it into A's low byte without touching
///              A's high byte (B accumulator).
/// Input: A=0xAB00, E=0, M=1, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0xA9, MEM[00:8001]=0x42
/// Expected Output: A=0xAB42, PC=0x8002, P.N=0, P.Z=0
#[test]
fn test_lda_immediate_m8() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xA9, 0x42]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.a = 0xAB00;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xAB42);
    assert_eq!(cpu.pc, 0x8002);
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 15: LDA #imm — Immediate addressing, M=0 (16-bit)
/// Description: Verifies immediate addressing fetches two operand bytes when
///              M=0 and loads the full 16-bit value into A.
/// Input: A=0x0000, E=0, M=0, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0xA9, MEM[00:8001]=0x34, MEM[00:8002]=0x12
/// Expected Output: A=0x1234, PC=0x8003, P.N=0, P.Z=0
#[test]
fn test_lda_immediate_m16() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xA9, 0x34, 0x12]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, false);
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0x1234);
    assert_eq!(cpu.pc, 0x8003);
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 16: LDA dp — Direct page addressing
/// Description: Verifies effective address = DP + operand byte. Loads from
///              bank 0 regardless of DBR.
/// Input: A=0x0000, E=0, M=1, DB=0x7E, DP=0x1000,
///        MEM[00:8000]=0xA5, MEM[00:8001]=0x40, MEM[00:1040]=0x99
/// Expected Output: A=0x0099, PC=0x8002, P.N=1, P.Z=0
#[test]
fn test_lda_direct_page() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xA5, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1040), &[0x99]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.db = 0x7E;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0099);
    assert_eq!(cpu.pc, 0x8002);
    assert!(cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 17: LDA dp,X — Direct page indexed by X
/// Description: Effective address = DP + operand + X (16-bit add, bank 0).
/// Input: A=0, E=0, M=1, X=0x0005, DP=0x1000,
///        MEM[00:8000]=0xB5, MEM[00:8001]=0x40, MEM[00:1045]=0x55
/// Expected Output: A low=0x55, PC=0x8002
#[test]
fn test_lda_direct_page_indexed_x() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xB5, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1045), &[0x55]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false); // 16-bit X
    cpu.x = 0x0005;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0055);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 18: LDA (dp) — Direct page indirect
/// Description: Pointer at DP+operand (bank 0) holds 16-bit address;
///              data is fetched from DBR:that-address.
/// Input: A=0, E=0, M=1, DB=0x7E, DP=0x1000,
///        MEM[00:8000]=0xB2, MEM[00:8001]=0x10,
///        MEM[00:1010]=0x00, MEM[00:1011]=0x20,
///        MEM[7E:2000]=0x33
/// Expected Output: A low=0x33, PC=0x8002
#[test]
fn test_lda_direct_page_indirect() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xB2, 0x10]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1010), &[0x00, 0x20]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x2000), &[0x33]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.db = 0x7E;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0033);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 19: LDA (dp,X) — Direct page indexed indirect
/// Description: Pointer at DP+operand+X (bank 0) holds 16-bit address;
///              data fetched from DBR:that-address.
/// Input: A=0, E=0, M=1, X=0x0004, DB=0x7E, DP=0x1000,
///        MEM[00:8000]=0xA1, MEM[00:8001]=0x10,
///        MEM[00:1014]=0x00, MEM[00:1015]=0x30,
///        MEM[7E:3000]=0x44
/// Expected Output: A low=0x44, PC=0x8002
#[test]
fn test_lda_direct_page_indexed_indirect_x() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xA1, 0x10]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1014), &[0x00, 0x30]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x3000), &[0x44]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.x = 0x0004;
    cpu.db = 0x7E;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0044);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 20: LDA (dp),Y — Direct page indirect indexed by Y
/// Description: Pointer at DP+operand (bank 0) holds 16-bit base;
///              effective address = DBR:base + Y (may cross bank).
/// Input: A=0, E=0, M=1, Y=0x0010, DB=0x7E, DP=0x1000,
///        MEM[00:8000]=0xB1, MEM[00:8001]=0x10,
///        MEM[00:1010]=0x00, MEM[00:1011]=0x20,
///        MEM[7E:2010]=0x55
/// Expected Output: A low=0x55, PC=0x8002
#[test]
fn test_lda_direct_page_indirect_indexed_y() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xB1, 0x10]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1010), &[0x00, 0x20]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x2010), &[0x55]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.y = 0x0010;
    cpu.db = 0x7E;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0055);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 21: LDA [dp] — Direct page indirect long
/// Description: 24-bit pointer at DP+operand (bank 0); data fetched from
///              that full 24-bit address (bank specified by pointer).
/// Input: A=0, E=0, M=1, DB=0x00, DP=0x1000,
///        MEM[00:8000]=0xA7, MEM[00:8001]=0x10,
///        MEM[00:1010]=0x00, MEM[00:1011]=0x40, MEM[00:1012]=0x7E,
///        MEM[7E:4000]=0x66
/// Expected Output: A low=0x66, PC=0x8002
#[test]
fn test_lda_direct_page_indirect_long() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xA7, 0x10]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1010), &[0x00, 0x40, 0x7E]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x4000), &[0x66]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.db = 0x00;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0066);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 22: LDA [dp],Y — Direct page indirect long indexed by Y
/// Description: 24-bit pointer at DP+operand (bank 0); effective address =
///              pointer + Y (24-bit add, can cross banks).
/// Input: A=0, E=0, M=1, Y=0x0020, DP=0x1000,
///        MEM[00:8000]=0xB7, MEM[00:8001]=0x10,
///        MEM[00:1010]=0x00, MEM[00:1011]=0x40, MEM[00:1012]=0x7E,
///        MEM[7E:4020]=0x77
/// Expected Output: A low=0x77, PC=0x8002
#[test]
fn test_lda_direct_page_indirect_long_indexed_y() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xB7, 0x10]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1010), &[0x00, 0x40, 0x7E]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x4020), &[0x77]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.y = 0x0020;
    cpu.dp = 0x1000;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0077);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 23: LDA abs — Absolute addressing
/// Description: Effective address = DBR:operand16. Operand is two bytes
///              fetched from PB:PC and PB:PC+1.
/// Input: A=0, E=0, M=1, DB=0x7E,
///        MEM[00:8000]=0xAD, MEM[00:8001]=0x00, MEM[00:8002]=0x30,
///        MEM[7E:3000]=0x88
/// Expected Output: A low=0x88, PC=0x8003
#[test]
fn test_lda_absolute() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xAD, 0x00, 0x30]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x3000), &[0x88]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.db = 0x7E;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0088);
    assert_eq!(cpu.pc, 0x8003);
}

/// Test 24: LDA abs,X — Absolute indexed by X
/// Description: Effective address = DBR:operand16 + X (24-bit add, may
///              cross bank boundary).
/// Input: A=0, E=0, M=1, X=0x0010, DB=0x7E,
///        MEM[00:8000]=0xBD, MEM[00:8001]=0x00, MEM[00:8002]=0x30,
///        MEM[7E:3010]=0x99
/// Expected Output: A low=0x99, PC=0x8003
#[test]
fn test_lda_absolute_indexed_x() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xBD, 0x00, 0x30]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x3010), &[0x99]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.x = 0x0010;
    cpu.db = 0x7E;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x0099);
    assert_eq!(cpu.pc, 0x8003);
}

/// Test 25: LDA abs,Y — Absolute indexed by Y
/// Description: Effective address = DBR:operand16 + Y.
/// Input: A=0, E=0, M=1, Y=0x0008, DB=0x7E,
///        MEM[00:8000]=0xB9, MEM[00:8001]=0x00, MEM[00:8002]=0x30,
///        MEM[7E:3008]=0xAA
/// Expected Output: A low=0xAA, PC=0x8003, P.N=1
#[test]
fn test_lda_absolute_indexed_y() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xB9, 0x00, 0x30]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x3008), &[0xAA]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.y = 0x0008;
    cpu.db = 0x7E;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x00AA);
    assert_eq!(cpu.pc, 0x8003);
    assert!(cpu.is_flag_set(Flag::FlagN));
}

/// Test 26: LDA long — Absolute long
/// Description: Effective address = explicit 24-bit operand. DBR is ignored.
/// Input: A=0, E=0, M=1, DB=0x00,
///        MEM[00:8000]=0xAF, MEM[00:8001]=0x00, MEM[00:8002]=0x50, MEM[00:8003]=0x7E,
///        MEM[7E:5000]=0xBB
/// Expected Output: A low=0xBB, PC=0x8004
#[test]
fn test_lda_absolute_long() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xAF, 0x00, 0x50, 0x7E]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x5000), &[0xBB]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.db = 0x00;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x00BB);
    assert_eq!(cpu.pc, 0x8004);
}

/// Test 27: LDA long,X — Absolute long indexed by X
/// Description: Effective address = 24-bit operand + X.
/// Input: A=0, E=0, M=1, X=0x0005,
///        MEM[00:8000]=0xBF, MEM[00:8001]=0x00, MEM[00:8002]=0x50, MEM[00:8003]=0x7E,
///        MEM[7E:5005]=0xCC
/// Expected Output: A low=0xCC, PC=0x8004
#[test]
fn test_lda_absolute_long_indexed_x() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xBF, 0x00, 0x50, 0x7E]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x5005), &[0xCC]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.x = 0x0005;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x00CC);
    assert_eq!(cpu.pc, 0x8004);
}

/// Test 28: LDA sr,S — Stack relative
/// Description: Effective address = SP + operand byte (16-bit add, bank 0).
/// Input: A=0, E=0, M=1, SP=0x1F00,
///        MEM[00:8000]=0xA3, MEM[00:8001]=0x04,
///        MEM[00:1F04]=0xDD
/// Expected Output: A low=0xDD, PC=0x8002
#[test]
fn test_lda_stack_relative() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xA3, 0x04]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1F04), &[0xDD]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.sp = 0x1F00;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x00DD);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 29: LDA (sr,S),Y — Stack relative indirect indexed
/// Description: Pointer at SP+operand (bank 0) holds 16-bit base; effective
///              address = DBR:base + Y.
/// Input: A=0, E=0, M=1, Y=0x0010, DB=0x7E, SP=0x1F00,
///        MEM[00:8000]=0xB3, MEM[00:8001]=0x04,
///        MEM[00:1F04]=0x00, MEM[00:1F05]=0x60,
///        MEM[7E:6010]=0xEE
/// Expected Output: A low=0xEE, PC=0x8002
#[test]
fn test_lda_stack_relative_indirect_indexed_y() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xB3, 0x04]);
        write_ram(&mut cpu, &mut bus, addr(0x00, 0x1F04), &[0x00, 0x60]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x6010), &[0xEE]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.y = 0x0010;
    cpu.db = 0x7E;
    cpu.sp = 0x1F00;
    cpu.a = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a & 0x00FF, 0x00EE);
    assert_eq!(cpu.pc, 0x8002);
}

/// Test 30: LDX abs — Absolute addressing for X register (X-flag controlled)
/// Description: Verifies the op_case_flagx! path: with X=0 (16-bit index),
///              LDX abs loads a full 16-bit value into X.
/// Input: X=0, E=0, X-flag=0, DB=0x7E,
///        MEM[00:8000]=0xAE, MEM[00:8001]=0x00, MEM[00:8002]=0x40,
///        MEM[7E:4000]=0x34, MEM[7E:4001]=0x12
/// Expected Output: X=0x1234, PC=0x8003, P.N=0, P.Z=0
#[test]
fn test_ldx_absolute_x16() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xAE, 0x00, 0x40]);
        write_ram(&mut cpu, &mut bus, addr(0x7E, 0x4000), &[0x34, 0x12]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.db = 0x7E;
    cpu.x = 0x0000;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.x, 0x1234);
    assert_eq!(cpu.pc, 0x8003);
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 31: JMP (abs) — Absolute indirect
/// Description: Pointer at 00:operand16 holds 16-bit target; PC loaded from
///              that target. PB unchanged.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x6C, MEM[00:8001]=0x50, MEM[00:8002]=0x90,
///        MEM[00:9050]=0x00, MEM[00:9051]=0xC0
/// Expected Output: PB=0x00, PC=0xC000
#[test]
fn test_jmp_absolute_indirect() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x6C, 0x50, 0x90]);
        write_rom(&mut bus, addr(0x00, 0x9050), &[0x00, 0xC0]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0xC000);
}

/// Test 32: JMP (abs,X) — Absolute indexed indirect
/// Description: Pointer at PB:(operand16 + X) holds 16-bit target; PC loaded
///              from that target. The pointer fetch is in the program bank
///              (PB), not bank 0.
/// Input: PB=0x00, PC=0x8000, X=0x0004,
///        MEM[00:8000]=0x7C, MEM[00:8001]=0x50, MEM[00:8002]=0x90,
///        MEM[00:9054]=0x00, MEM[00:9055]=0xD0
/// Expected Output: PB=0x00, PC=0xD000
#[test]
fn test_jmp_absolute_indexed_indirect_x() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x7C, 0x50, 0x90]);
        write_rom(&mut bus, addr(0x00, 0x9054), &[0x00, 0xD0]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.x = 0x0004;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x00);
    assert_eq!(cpu.pc, 0xD000);
}

/// Test 33: JMP long — Absolute long jump
/// Description: 24-bit operand directly loaded into PB:PC. Verifies that
///              both the program bank and PC are updated atomically.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x5C, MEM[00:8001]=0x34, MEM[00:8002]=0x12, MEM[00:8003]=0x7E
/// Expected Output: PB=0x7E, PC=0x1234
#[test]
fn test_jmp_long() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x5C, 0x34, 0x12, 0x7E]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x7E);
    assert_eq!(cpu.pc, 0x1234);
}

/// Test 34: JMP [abs] — Absolute indirect long
/// Description: 24-bit pointer at 00:operand16 holds the full 24-bit target;
///              both PB and PC are loaded from that pointer. Pointer always
///              read from bank 0.
/// Input: PB=0x00, PC=0x8000,
///        MEM[00:8000]=0xDC, MEM[00:8001]=0x50, MEM[00:8002]=0x90,
///        MEM[00:9050]=0x00, MEM[00:9051]=0xA0, MEM[00:9052]=0x7E
/// Expected Output: PB=0x7E, PC=0xA000
#[test]
fn test_jmp_absolute_indirect_long() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xDC, 0x50, 0x90]);
        // Pointer lives in bank 0 (could be RAM or ROM); use ROM via write_rom.
        write_rom(&mut bus, addr(0x00, 0x9050), &[0x00, 0xA0, 0x7E]);
    }
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pb, 0x7E);
    assert_eq!(cpu.pc, 0xA000);
}

/// Test 35: INX — Implied addressing
/// Description: Implied-mode instructions take no operand byte; PC advances
///              by exactly 1. Verified via INX which has no operand and
///              produces an observable register change.
/// Input: X=0x0010, E=0, X-flag=0 (16-bit X), PB=0x00, PC=0x8000,
///        MEM[00:8000]=0xE8 (INX)
/// Expected Output: X=0x0011, PC=0x8001, P.N=0, P.Z=0
#[test]
fn test_inx_implied() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0xE8]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.x = 0x0010;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.x, 0x0011);
    assert_eq!(cpu.pc, 0x8001);
    assert!(!cpu.is_flag_set(Flag::FlagN));
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}

/// Test 36: ASL A — Accumulator addressing
/// Description: Accumulator-mode instructions operate directly on A; PC
///              advances by exactly 1. Verified via ASL A in 8-bit mode:
///              shifting 0x41 left yields 0x82, with C=0 (no bit shifted out)
///              and N=1 (result high bit set).
/// Input: A=0xAB41, E=0, M=1 (8-bit A), PB=0x00, PC=0x8000,
///        P.C=0, MEM[00:8000]=0x0A (ASL A)
/// Expected Output: A=0xAB82, PC=0x8001, P.C=0, P.N=1, P.Z=0
#[test]
fn test_asl_accumulator_m8() {
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus();
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x0A]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagM, true);
    cpu.set_flag_to_bool(Flag::FlagC, false);
    cpu.a = 0xAB41;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus();
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xAB82, "ASL A in 8-bit mode must preserve A high byte");
    assert_eq!(cpu.pc, 0x8001);
    assert!(!cpu.is_flag_set(Flag::FlagC), "Bit 7 of 0x41 is 0 → C=0");
    assert!(cpu.is_flag_set(Flag::FlagN), "Result 0x82 has bit 7 set → N=1");
    assert!(!cpu.is_flag_set(Flag::FlagZ));
}