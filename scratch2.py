f = open("headers.txt", "r")
headers = f.readlines()
f.close()

f = open("opcodes.txt")
opcodes = f.readlines()
f.close()

addr_mode_map = {
    "abs": "self.absolute[[SIZE]]()",
    "long": "self.absolute"
    ""
    "imp": "",
    "imm": "self.immediate[[SIZE]]()",
    "dir": "self.direct[[SIZE]]()",
    const AddressingMode AbsoluteLong         = 1;
    const AddressingMode AbsoluteLongX        = 2;
    const AddressingMode AbsoluteX            = 3;
    const AddressingMode AbsoluteY            = 4;
    const AddressingMode AbsoluteIndirect     = 5;
    const AddressingMode AbsoluteIndirectLong = 6;
    const AddressingMode AbsoluteXIndirect    = 7;
    const AddressingMode Direct               = 8;
    const AddressingMode DirectX              = 9;
    const AddressingMode DirectY              = 10;
    const AddressingMode DirectIndirect       = 11;
    const AddressingMode DirectIndirectLong   = 12;
    const AddressingMode DirectXIndirect      = 13;
    const AddressingMode DirectIndirectY      = 14;
    const AddressingMode DirectIndirectLongY  = 15;
    const AddressingMode Immediate            = 16;
    const AddressingMode Implied              = 17;
    const AddressingMode Accumulator          = 18;
    const AddressingMode Relative8            = 19;
    const AddressingMode Relative16           = 20;
    const AddressingMode SourceDestination    = 21;
    const AddressingMode Stack                = 22;
    const AddressingMode StackIndirectY       = 23;
}

mode_case_map = {
    "all": "([[OPCODE]], ..)",
    "n": "([[OPCODE]], CpuMode::Native, ..)",
    "e": "([[OPCODE]], CpuMode::Emulation, ..)",
    "m8": "([[OPCODE]], _, RegMode::Byte, _)",
    "m16": "([[OPCODE]], _, RegMode::TwoBytes, _)",
    "x8": "([[OPCODE]], _, _, RegMode::Byte)",
    "x16": "([[OPCODE]], _, _, RegMode::TwoBytes)",

    "acc_m8": "([[OPCODE]], _, RegMode::Byte, _)",
    "acc_m16": "([[OPCODE]], _, RegMode::TwoBytes, _)",
    "mem_m8": "([[OPCODE]], _, RegMode::Byte, _)",
    "mem_m16": "([[OPCODE]], _, RegMode::TwoBytes, _)",
}

f = open("match.txt", "w")

for opcode_line in opcodes:
    opcode_data = opcode_line.split()

    opcode = opcode_data[0]
    instr_name = opcode_data[1].lower()
    addr_mode = opcode_data[2]

    func_headers = []
    for header in headers:
        if header[:3] == instr_name:
            func_headers.append(header)
    
    if func_headers == []:
        print(f"No headers found for instr '{instr_name}' (OP: {opcode})")
        break
    
    modes = []
    for header in func_headers:
        modes.append(header[4:header.find("(")])

    case_str = f"// {instr_name}, {addr_mode}\n"

    # f.write(f"// {instr_name}\n")

    for mode in modes:
        case_str += mode_case_map[mode] + " => {\n"


    
    case_str = case_str.replace("[[OPCODE]]", opcode)

    print(case_str)
