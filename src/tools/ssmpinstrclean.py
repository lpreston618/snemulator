a = """Instruction{opcode: 0x00, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // NOP
Instruction{opcode: 0x01, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x02, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0x03, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0x04, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // ORA
Instruction{opcode: 0x05, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // ORA
Instruction{opcode: 0x06, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // ORA
Instruction{opcode: 0x07, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // ORA
Instruction{opcode: 0x08, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // ORA
Instruction{opcode: 0x09, cycles:  6, addr_mode: AddressingMode::DirectToDirect, branching: false},   // ORA
Instruction{opcode: 0x0A, cycles:  5, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // OR1
Instruction{opcode: 0x0B, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // ASL
Instruction{opcode: 0x0C, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // ASL
Instruction{opcode: 0x0D, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // PUSH
Instruction{opcode: 0x0E, cycles:  6, addr_mode: AddressingMode::Absolute, branching: false},   // TSET1
Instruction{opcode: 0x0F, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // BRK
Instruction{opcode: 0x10, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BPL
Instruction{opcode: 0x11, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x12, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0x13, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0x14, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // ORA
Instruction{opcode: 0x15, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // ORA
Instruction{opcode: 0x16, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // ORA
Instruction{opcode: 0x17, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // ORA
Instruction{opcode: 0x18, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // OR
Instruction{opcode: 0x19, cycles:  5, addr_mode: AddressingMode::IndirectToIndirect, branching: false},   // OR
Instruction{opcode: 0x1A, cycles:  6, addr_mode: AddressingMode::DirectWord, branching: false},   // DECW
Instruction{opcode: 0x1B, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // ASL
Instruction{opcode: 0x1C, cycles:  2, addr_mode: AddressingMode::Accumulator, branching: false},   // ASL
Instruction{opcode: 0x1D, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // DEX
Instruction{opcode: 0x1E, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // CPX
Instruction{opcode: 0x1F, cycles:  6, addr_mode: AddressingMode::Absolute, branching: false},   // JMP
Instruction{opcode: 0x20, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // CLRP
Instruction{opcode: 0x21, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x22, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0x23, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0x24, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // AND
Instruction{opcode: 0x25, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // AND
Instruction{opcode: 0x26, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // AND
Instruction{opcode: 0x27, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // AND
Instruction{opcode: 0x28, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // AND
Instruction{opcode: 0x29, cycles:  6, addr_mode: AddressingMode::DirectToDirect, branching: false},   // AND
Instruction{opcode: 0x2A, cycles:  5, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // OR1
Instruction{opcode: 0x2B, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // ROL
Instruction{opcode: 0x2C, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // ROL
Instruction{opcode: 0x2D, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // PUSH
Instruction{opcode: 0x2E, cycles:  5, addr_mode: AddressingMode::Relative, branching:  true},   // CBNE
Instruction{opcode: 0x2F, cycles:  4, addr_mode: AddressingMode::Relative, branching: false},   // BRA
Instruction{opcode: 0x30, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BMI
Instruction{opcode: 0x31, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x32, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0x33, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0x34, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // AND
Instruction{opcode: 0x35, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // AND
Instruction{opcode: 0x36, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // AND
Instruction{opcode: 0x37, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // AND
Instruction{opcode: 0x38, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // AND
Instruction{opcode: 0x39, cycles:  5, addr_mode: AddressingMode::IndirectToIndirect, branching: false},   // AND
Instruction{opcode: 0x3A, cycles:  6, addr_mode: AddressingMode::DirectWord, branching: false},   // INCW
Instruction{opcode: 0x3B, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // ROL
Instruction{opcode: 0x3C, cycles:  2, addr_mode: AddressingMode::Accumulator, branching: false},   // ROL
Instruction{opcode: 0x3D, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // INX
Instruction{opcode: 0x3E, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // CPX
Instruction{opcode: 0x3F, cycles:  8, addr_mode: AddressingMode::Absolute, branching: false},   // CALL
Instruction{opcode: 0x40, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // SETP
Instruction{opcode: 0x41, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x42, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0x43, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0x44, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // EOR
Instruction{opcode: 0x45, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // EOR
Instruction{opcode: 0x46, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // EOR
Instruction{opcode: 0x47, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // EOR
Instruction{opcode: 0x48, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // EOR
Instruction{opcode: 0x49, cycles:  6, addr_mode: AddressingMode::DirectToDirect, branching: false},   // EOR
Instruction{opcode: 0x4A, cycles:  4, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // AND1
Instruction{opcode: 0x4B, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // LSR
Instruction{opcode: 0x4C, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // LSR
Instruction{opcode: 0x4D, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // PUSH
Instruction{opcode: 0x4E, cycles:  6, addr_mode: AddressingMode::Absolute, branching: false},   // TCLR1
Instruction{opcode: 0x4F, cycles:  6, addr_mode: AddressingMode::Implied, branching: false},   // PCALL
Instruction{opcode: 0x50, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BVC
Instruction{opcode: 0x51, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x52, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0x53, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0x54, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // EOR
Instruction{opcode: 0x55, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // EOR
Instruction{opcode: 0x56, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // EOR
Instruction{opcode: 0x57, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // EOR
Instruction{opcode: 0x58, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // EOR
Instruction{opcode: 0x59, cycles:  5, addr_mode: AddressingMode::IndirectToIndirect, branching: false},   // EOR
Instruction{opcode: 0x5A, cycles:  4, addr_mode: AddressingMode::DirectWord, branching: false},   // CMPW
Instruction{opcode: 0x5B, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // LSR
Instruction{opcode: 0x5C, cycles:  2, addr_mode: AddressingMode::Accumulator, branching: false},   // LSR
Instruction{opcode: 0x5D, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // LDX
Instruction{opcode: 0x5E, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // CPY
Instruction{opcode: 0x5F, cycles:  3, addr_mode: AddressingMode::Absolute, branching: false},   // JMP
Instruction{opcode: 0x60, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // CLRC
Instruction{opcode: 0x61, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x62, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0x63, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0x64, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // CMP
Instruction{opcode: 0x65, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // CMP
Instruction{opcode: 0x66, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // CMP
Instruction{opcode: 0x67, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // CMP
Instruction{opcode: 0x68, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // CMP
Instruction{opcode: 0x69, cycles:  6, addr_mode: AddressingMode::DirectToDirect, branching: false},   // CMP
Instruction{opcode: 0x6A, cycles:  4, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // AND1
Instruction{opcode: 0x6B, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // ROR
Instruction{opcode: 0x6C, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // ROR
Instruction{opcode: 0x6D, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // PUSH
Instruction{opcode: 0x6E, cycles:  5, addr_mode: AddressingMode::Relative, branching:  true},   // DBNZ
Instruction{opcode: 0x6F, cycles:  5, addr_mode: AddressingMode::Implied, branching: false},   // RET
Instruction{opcode: 0x70, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BVS
Instruction{opcode: 0x71, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x72, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0x73, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0x74, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // CMP
Instruction{opcode: 0x75, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // CMP
Instruction{opcode: 0x76, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // CMP
Instruction{opcode: 0x77, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // CMP
Instruction{opcode: 0x78, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // CMP
Instruction{opcode: 0x79, cycles:  5, addr_mode: AddressingMode::IndirectToIndirect, branching: false},   // CMP
Instruction{opcode: 0x7A, cycles:  5, addr_mode: AddressingMode::DirectWord, branching: false},   // ADDW
Instruction{opcode: 0x7B, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // ROR
Instruction{opcode: 0x7C, cycles:  2, addr_mode: AddressingMode::Accumulator, branching: false},   // ROR
Instruction{opcode: 0x7D, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // LDA
Instruction{opcode: 0x7E, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // CPY
Instruction{opcode: 0x7F, cycles:  6, addr_mode: AddressingMode::Implied, branching: false},   // RET1
Instruction{opcode: 0x80, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // SETC
Instruction{opcode: 0x81, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x82, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0x83, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0x84, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // ADC
Instruction{opcode: 0x85, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // ADC
Instruction{opcode: 0x86, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // ADC
Instruction{opcode: 0x87, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // ADC
Instruction{opcode: 0x88, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // ADC
Instruction{opcode: 0x89, cycles:  6, addr_mode: AddressingMode::DirectToDirect, branching: false},   // ADC
Instruction{opcode: 0x8A, cycles:  5, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // EOR1
Instruction{opcode: 0x8B, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // DEC
Instruction{opcode: 0x8C, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // DEC
Instruction{opcode: 0x8D, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // LDY
Instruction{opcode: 0x8E, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // POP
Instruction{opcode: 0x8F, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // MOV
Instruction{opcode: 0x90, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BCC
Instruction{opcode: 0x91, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0x92, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0x93, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0x94, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // ADC
Instruction{opcode: 0x95, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // ADC
Instruction{opcode: 0x96, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // ADC
Instruction{opcode: 0x97, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // ADC
Instruction{opcode: 0x98, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // ADC
Instruction{opcode: 0x99, cycles:  5, addr_mode: AddressingMode::IndirectToIndirect, branching: false},   // ADC
Instruction{opcode: 0x9A, cycles:  5, addr_mode: AddressingMode::DirectWord, branching: false},   // SUBW
Instruction{opcode: 0x9B, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // DEC
Instruction{opcode: 0x9C, cycles:  2, addr_mode: AddressingMode::Accumulator, branching: false},   // DEC
Instruction{opcode: 0x9D, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // LDX
Instruction{opcode: 0x9E, cycles: 12, addr_mode: AddressingMode::Implied, branching: false},   // DIV
Instruction{opcode: 0x9F, cycles:  5, addr_mode: AddressingMode::Accumulator, branching: false},   // XCN
Instruction{opcode: 0xA0, cycles:  3, addr_mode: AddressingMode::Implied, branching: false},   // SEI
Instruction{opcode: 0xA1, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0xA2, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0xA3, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0xA4, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // SBC
Instruction{opcode: 0xA5, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // SBC
Instruction{opcode: 0xA6, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // SBC
Instruction{opcode: 0xA7, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // SBC
Instruction{opcode: 0xA8, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // SBC
Instruction{opcode: 0xA9, cycles:  6, addr_mode: AddressingMode::DirectToDirect, branching: false},   // SBC
Instruction{opcode: 0xAA, cycles:  4, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // MOV1
Instruction{opcode: 0xAB, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // INC
Instruction{opcode: 0xAC, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // INC
Instruction{opcode: 0xAD, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // CPY
Instruction{opcode: 0xAE, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // POP
Instruction{opcode: 0xAF, cycles:  4, addr_mode: AddressingMode::IndirectInc, branching: false},   // STA
Instruction{opcode: 0xB0, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BCS
Instruction{opcode: 0xB1, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0xB2, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0xB3, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0xB4, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // SBC
Instruction{opcode: 0xB5, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // SBC
Instruction{opcode: 0xB6, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // SBC
Instruction{opcode: 0xB7, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // SBC
Instruction{opcode: 0xB8, cycles:  5, addr_mode: AddressingMode::ImmediateToDirect, branching: false},   // SBC
Instruction{opcode: 0xB9, cycles:  5, addr_mode: AddressingMode::IndirectToIndirect, branching: false},   // SBC
Instruction{opcode: 0xBA, cycles:  5, addr_mode: AddressingMode::DirectWord, branching: false},   // LDYA
Instruction{opcode: 0xBB, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // INC
Instruction{opcode: 0xBC, cycles:  2, addr_mode: AddressingMode::Accumulator, branching: false},   // INC
Instruction{opcode: 0xBD, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // STX
Instruction{opcode: 0xBE, cycles:  3, addr_mode: AddressingMode::Implied, branching: false},   // DAS
Instruction{opcode: 0xBF, cycles:  4, addr_mode: AddressingMode::IndirectInc, branching: false},   // LDA
Instruction{opcode: 0xC0, cycles:  3, addr_mode: AddressingMode::Implied, branching: false},   // CLI
Instruction{opcode: 0xC1, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0xC2, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0xC3, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0xC4, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // STA
Instruction{opcode: 0xC5, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // STA
Instruction{opcode: 0xC6, cycles:  4, addr_mode: AddressingMode::Indirect, branching: false},   // STA
Instruction{opcode: 0xC7, cycles:  7, addr_mode: AddressingMode::XIndirect, branching: false},   // STA
Instruction{opcode: 0xC8, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // CPX
Instruction{opcode: 0xC9, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // STX
Instruction{opcode: 0xCA, cycles:  6, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // MOV1
Instruction{opcode: 0xCB, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // STY
Instruction{opcode: 0xCC, cycles:  5, addr_mode: AddressingMode::Absolute, branching: false},   // STY
Instruction{opcode: 0xCD, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // LDX
Instruction{opcode: 0xCE, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // POP
Instruction{opcode: 0xCF, cycles:  9, addr_mode: AddressingMode::Implied, branching: false},   // MUL
Instruction{opcode: 0xD0, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BNE
Instruction{opcode: 0xD1, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0xD2, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0xD3, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0xD4, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // STA
Instruction{opcode: 0xD5, cycles:  6, addr_mode: AddressingMode::XAbsolute, branching: false},   // STA
Instruction{opcode: 0xD6, cycles:  6, addr_mode: AddressingMode::YAbsolute, branching: false},   // STA
Instruction{opcode: 0xD7, cycles:  7, addr_mode: AddressingMode::IndirectY, branching: false},   // STA
Instruction{opcode: 0xD8, cycles:  4, addr_mode: AddressingMode::Direct, branching: false},   // STX
Instruction{opcode: 0xD9, cycles:  5, addr_mode: AddressingMode::YDirect, branching: false},   // STX
Instruction{opcode: 0xDA, cycles:  5, addr_mode: AddressingMode::DirectWord, branching: false},   // STYA
Instruction{opcode: 0xDB, cycles:  5, addr_mode: AddressingMode::XDirect, branching: false},   // STY
Instruction{opcode: 0xDC, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // DEY
Instruction{opcode: 0xDD, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // LDA
Instruction{opcode: 0xDE, cycles:  6, addr_mode: AddressingMode::XDirect, branching:  true},   // CBNE
Instruction{opcode: 0xDF, cycles:  3, addr_mode: AddressingMode::Implied, branching: false},   // DAA
Instruction{opcode: 0xE0, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // CLRV
Instruction{opcode: 0xE1, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0xE2, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // SET1
Instruction{opcode: 0xE3, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBS
Instruction{opcode: 0xE4, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // LDA
Instruction{opcode: 0xE5, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // LDA
Instruction{opcode: 0xE6, cycles:  3, addr_mode: AddressingMode::Indirect, branching: false},   // LDA
Instruction{opcode: 0xE7, cycles:  6, addr_mode: AddressingMode::XIndirect, branching: false},   // LDA
Instruction{opcode: 0xE8, cycles:  2, addr_mode: AddressingMode::Immediate, branching: false},   // LDA
Instruction{opcode: 0xE9, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // LDX
Instruction{opcode: 0xEA, cycles:  5, addr_mode: AddressingMode::AbsoluteBit, branching: false},   // NOT1
Instruction{opcode: 0xEB, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // LDY
Instruction{opcode: 0xEC, cycles:  4, addr_mode: AddressingMode::Absolute, branching: false},   // LDY
Instruction{opcode: 0xED, cycles:  3, addr_mode: AddressingMode::Implied, branching: false},   // NOTC
Instruction{opcode: 0xEE, cycles:  4, addr_mode: AddressingMode::Implied, branching: false},   // POP
Instruction{opcode: 0xEF, cycles:  0, addr_mode: AddressingMode::Implied, branching: false},   // SLEEP
Instruction{opcode: 0xF0, cycles:  2, addr_mode: AddressingMode::Relative, branching:  true},   // BEQ
Instruction{opcode: 0xF1, cycles:  8, addr_mode: AddressingMode::Implied, branching: false},   // TCALL
Instruction{opcode: 0xF2, cycles:  4, addr_mode: AddressingMode::DirectBit, branching: false},   // CLR1
Instruction{opcode: 0xF3, cycles:  5, addr_mode: AddressingMode::DirectBitRelative, branching:  true},   // BBC
Instruction{opcode: 0xF4, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // LDA
Instruction{opcode: 0xF5, cycles:  5, addr_mode: AddressingMode::XAbsolute, branching: false},   // LDA
Instruction{opcode: 0xF6, cycles:  5, addr_mode: AddressingMode::YAbsolute, branching: false},   // LDA
Instruction{opcode: 0xF7, cycles:  6, addr_mode: AddressingMode::IndirectY, branching: false},   // LDA
Instruction{opcode: 0xF8, cycles:  3, addr_mode: AddressingMode::Direct, branching: false},   // LDX
Instruction{opcode: 0xF9, cycles:  4, addr_mode: AddressingMode::YDirect, branching: false},   // LDX
Instruction{opcode: 0xFA, cycles:  5, addr_mode: AddressingMode::DirectToDirect, branching: false},   // MOV
Instruction{opcode: 0xFB, cycles:  4, addr_mode: AddressingMode::XDirect, branching: false},   // LDY
Instruction{opcode: 0xFC, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // INY
Instruction{opcode: 0xFD, cycles:  2, addr_mode: AddressingMode::Implied, branching: false},   // LDY
Instruction{opcode: 0xFE, cycles:  4, addr_mode: AddressingMode::Relative, branching:  true},   // DBNZ
Instruction{opcode: 0xFF, cycles:  0, addr_mode: AddressingMode::Implied, branching: false},   // STOP"""

bytes_map = {
    "Direct": 2,
    "DirectWord": 3,
    "XDirect": 2,
    "YDirect": 2,
    "Indirect": 1,
    "IndirectInc": 1,
    "DirectToDirect": 3,
    "IndirectToIndirect": 1,
    "ImmediateToDirect": 3,
    "DirectBit": 2,
    "DirectBitRelative": 3,
    "AbsoluteBit": 3,
    "Absolute": 3,
    "AbsoluteXIndirect": 3,
    "XAbsolute": 3,
    "YAbsolute": 3,
    "XIndirect": 2,
    "IndirectY": 2,
    "Relative": 2,
    "Immediate": 2,
    "Accumulator": 1,
    "Implied": 1,
}

num_addrs_map = {
    "Direct": 1,
    "DirectWord": 2,
    "XDirect": 1,
    "YDirect": 1,
    "Indirect": 1,
    "IndirectInc": 1,
    "DirectToDirect": 2,
    "IndirectToIndirect": 2,
    "ImmediateToDirect": 2,
    "DirectBit": 1,
    "DirectBitRelative": 2,
    "AbsoluteBit": 1,
    "Absolute": 1,
    "AbsoluteXIndirect": 1,
    "XAbsolute": 1,
    "YAbsolute": 1,
    "XIndirect": 1,
    "IndirectY": 1,
    "Relative": 1,
    "Immediate": 1,
    "Accumulator": 0,
    "Implied": 0,
}

def addr_mode_func_str(addr_mode):
    result = ""
    for i, char in enumerate(addr_mode):
        if char.upper() == char:
            if i != 0:
                result += "_"
            result += char.lower()
        else:
            result += char
    return result

def into_case(opcode, instr_name, addr_mode, cycles):
    addr_func_str = addr_mode_func_str(addr_mode)
    instr_bytes = bytes_map[addr_mode]
    instr_func_str = instr_name.lower()
    num_addrs = num_addrs_map[addr_mode]

    result =  f"{opcode} => " + "{\n"

    if num_addrs == 1:
        result += f"    let addr = self.{addr_func_str}();\n"
    elif num_addrs == 2:
        result += f"    let (addr1, addr2) = self.{addr_func_str}();\n"

    result += f"    self.pc += {instr_bytes};\n"

    if num_addrs == 0:
        result += f"    self.{instr_func_str}();\n"
    elif num_addrs == 1:
        result += f"    self.{instr_func_str}(addr);\n"
    elif num_addrs == 3:
        result += f"    self.{instr_func_str}(addr1, addr2);\n"

    result += f"    cycles = {cycles};\n"
    result +=  "},\n"

    return result


if __name__ == "__main__":
    print("Generating Match-Case Statement...")

    lines = a.split('\n')
    result = ""
    for line in lines:
        split = line.split()

        opcode = line[20:24]
        instr_name = split[-1]
        addr_mode = split[5][16:-1]
        cycles = split[3][:-1]

        result += into_case(opcode, instr_name, addr_mode, cycles)


    with open("spc700_match.txt", "w") as f:
        f.write(result)

    print("Done.")