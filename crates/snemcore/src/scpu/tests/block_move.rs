//! block_move.rs — 65c816 CPU MVN/MVP instruction verification suite.
//!
//! Coverage in this file:
//!   * MVN / MVP: Block move instructions for single & multi-byte transfers.
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

#![allow(unused_imports)]

use crate::{debug::NullHarness, scpu::*};
use super::common::*;

/// Test 37: MVN — Block move (source/destination bank addressing)
/// Description: MVN takes two bank operands: destination bank then source
///              bank (in opcode-byte order). It moves one byte per execute()
///              call from src_bank:X to dst_bank:Y, increments X and Y,
///              decrements A, and sets DBR to the destination bank. PC
///              advances only when A wraps from 0x0000 to 0xFFFF (move
///              complete); otherwise PC stays at the MVN opcode so the next
///              execute() repeats the move.
///
///              This test verifies a single-byte step: A=0x0000 means
///              "move 1 byte" (count = A+1). After the step A becomes
///              0xFFFF, X and Y increment by 1, DBR is set to the
///              destination bank, the byte is copied, and PC advances
///              past the 3-byte instruction.
/// Input: A=0x0000, X=0x1000, Y=0x2000, E=0, X-flag=0,
///        DB=0x00, PB=0x00, PC=0x8000,
///        MEM[00:8000]=0x54 (MVN), MEM[00:8001]=0x7E (dst bank),
///        MEM[00:8002]=0x7F (src bank),
///        MEM[7F:1000]=0xA5
/// Expected Output: A=0xFFFF, X=0x1001, Y=0x2001, DB=0x7E,
///                  PC=0x8003, MEM[7E:2000]=0xA5
#[test]
fn test_mvn_block_move_single_byte() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x54, 0x7E, 0x7F]);
        write_ram(&mut cpu, &mut bus, addr(0x7F, 0x1000), &[0xA5]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, false); // 16-bit X/Y
    cpu.a = 0x0000; // count = 1
    cpu.x = 0x1000;
    cpu.y = 0x2000;
    cpu.db = 0x00;
    set_pc(&mut cpu, 0x00, 0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.a, 0xFFFF, "A must wrap to 0xFFFF after final byte");
    assert_eq!(cpu.x, 0x1001, "X must be incremented");
    assert_eq!(cpu.y, 0x2001, "Y must be incremented");
    assert_eq!(cpu.db, 0x7E, "DBR must be set to destination bank");
    assert_eq!(cpu.pc, 0x8003, "PC advances past MVN once A wraps");
    let mut bus = backing.bus(&mut harness);
    assert_eq!(
        cpu.read(&mut bus, addr(0x7E, 0x2000)),
        0xA5,
        "Byte must be copied to dst_bank:Y"
    );
}

/// Test 38: MVN — Multi-byte block move requires repeated execute() calls
/// Description: With A=0x0002 (count = 3), the first two execute() calls
///              must NOT advance PC past the MVN opcode (so the CPU keeps
///              re-executing it), and the third call (which causes A to
///              wrap) advances PC. This pins down the per-byte stepping
///              behavior of MVN.
/// Input: A=0x0002, X=0x1000, Y=0x2000, E=0, X-flag=0,
///        MEM[00:8000]=0x54, MEM[00:8001]=0x7E, MEM[00:8002]=0x7F,
///        MEM[7F:1000..1003]=[0x11, 0x22, 0x33]
/// Expected Output (after 3 execute() calls):
///        A=0xFFFF, X=0x1003, Y=0x2003, PC=0x8003,
///        MEM[7E:2000]=0x11, MEM[7E:2001]=0x22, MEM[7E:2002]=0x33
///        After call 1: PC=0x8000 (still on MVN)
///        After call 2: PC=0x8000 (still on MVN)
///        After call 3: PC=0x8003 (advanced)
#[test]
fn test_mvn_block_move_multi_byte_steps() {
    let mut harness = NullHarness {};
    let (mut cpu, mut backing) = mk_cpu_and_backing(0x8000);
    {
        let mut bus = backing.bus(&mut harness);
        cpu.reset(&mut bus);
        write_rom(&mut bus, addr(0x00, 0x8000), &[0x54, 0x7E, 0x7F]);
        write_ram(&mut cpu, &mut bus, addr(0x7F, 0x1000), &[0x11, 0x22, 0x33]);
    }
    cpu.e = false;
    cpu.set_flag_to_bool(Flag::FlagX, false);
    cpu.a = 0x0002; // count = 3
    cpu.x = 0x1000;
    cpu.y = 0x2000;
    cpu.db = 0x00;
    set_pc(&mut cpu, 0x00, 0x8000);

    // Step 1
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pc, 0x8000, "After step 1 PC should remain on MVN opcode");
    assert_eq!(cpu.a, 0x0001);
    assert_eq!(cpu.x, 0x1001);
    assert_eq!(cpu.y, 0x2001);

    // Step 2
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pc, 0x8000, "After step 2 PC should remain on MVN opcode");
    assert_eq!(cpu.a, 0x0000);
    assert_eq!(cpu.x, 0x1002);
    assert_eq!(cpu.y, 0x2002);

    // Step 3 (final — A wraps to 0xFFFF, PC advances)
    {
        let mut bus = backing.bus(&mut harness);
        cpu.execute(&mut bus);
    }
    assert_eq!(cpu.pc, 0x8003, "After final step PC must advance past MVN");
    assert_eq!(cpu.a, 0xFFFF);
    assert_eq!(cpu.x, 0x1003);
    assert_eq!(cpu.y, 0x2003);
    assert_eq!(cpu.db, 0x7E);

    let mut bus = backing.bus(&mut harness);
    assert_eq!(cpu.read(&mut bus, addr(0x7E, 0x2000)), 0x11);
    assert_eq!(cpu.read(&mut bus, addr(0x7E, 0x2001)), 0x22);
    assert_eq!(cpu.read(&mut bus, addr(0x7E, 0x2002)), 0x33);
}
