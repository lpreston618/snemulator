f = open("headers.txt", "r")
headers = f.readlines()
f.close()

f = open("opcodes.txt")
opcodes = f.readlines()
f.close()

addr_mode_map = {
    "abs": "self.absolute[[SIZE]]()",
    "long": "self.absolute[[SIZE]]()",
    "long,X": "self.absolute_long_x[[SIZE]]()",
    "abs,X": "self.absolute_x[[SIZE]]()",
    "abs,Y": "self.absolute_y[[SIZE]]()",
    "(abs)": "self.absolute_indirect[[SIZE]]()",
    "[abs]": "self.absolute_indirect_long[[SIZE]]()",
    "(abs,X)": "self.absolute_x_indirect[[SIZE]]()",

    "dir": "self.direct[[SIZE]]()",
    "dir,X": "self.direct_x[[SIZE]]()",
    "dir,Y": "self.direct_y[[SIZE]]()",
    "(dir)": "self.direct_indirect[[SIZE]]()",
    "[dir]": "self.direct_indirect_long[[SIZE]]()",
    "(dir,X)": "self.direct_x_indirect[[SIZE]]()",
    "(dir),Y": "self.direct_indirect_y[[SIZE]]()",
    "[dir],Y": "self.direct_indirect_long_y[[SIZE]]()",

    "imm": "self.immediate[[SIZE]]()",
    "imp": "",
    "acc": "",
    "rel8": "self.relative8()",
    "rel16": "self.relative16()",
    "src,dest": "self.src_dst()",
    "stk,S": "self.stack_s[[SIZE]]()",
    "(stk,S),Y": "self.stack_indirect_y[[SIZE]]()"
}

mode_case_map = {
    "all": "([[OPCODE]], ..)",
    "n": "([[OPCODE]], CpuMode::Native, ..)",
    "e": "([[OPCODE]], CpuMode::Emulation, ..)",
    "m8": "([[OPCODE]], _, RegSize::Byte, _)",
    "m16": "([[OPCODE]], _, RegSize::TwoBytes, _)",
    "x8": "([[OPCODE]], _, _, RegSize::Byte)",
    "x16": "([[OPCODE]], _, _, RegSize::TwoBytes)",

    "acc_m8": "([[OPCODE]], _, RegSize::Byte, _)",
    "acc_m16": "([[OPCODE]], _, RegSize::TwoBytes, _)",
    "mem_m8": "([[OPCODE]], _, RegSize::Byte, _)",
    "mem_m16": "([[OPCODE]], _, RegSize::TwoBytes, _)",
}

f = open("match.txt", "w")

for opcode_line in opcodes:
    opcode_data = opcode_line.split()

    opcode = opcode_data[0]
    instr_name = opcode_data[1].lower()
    addr_mode = opcode_data[2]
    instr_len = int(opcode_data[3][0])
    instr_len_m_diff = "-m" in opcode_data[3]
    instr_len_x_diff = "-x" in opcode_data[3]

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

    for i, mode in enumerate(modes):
        if "acc" in mode and addr_mode != "acc":
            # modes.remove(mode)
            modes[i] = None
        
        elif "mem" in mode and addr_mode == "acc":
            # modes.remove(mode)
            modes[i] = None
    
    while None in modes:
        modes.remove(None)

    case_str = f"// {instr_name}, {addr_mode}\n"

    for mode in modes:
        case_str += mode_case_map[mode] + " => {\n"

        addr_mode_func = addr_mode_map[addr_mode]

        if addr_mode != "imp" and addr_mode != "acc":
            case_str += f"    let addr = {addr_mode_func};\n"
            case_str += f"    self.{instr_name}_{mode}(addr);\n"
        else:
            case_str += f"    self.{instr_name}_{mode}();\n"
        
        instr_len_str = str(instr_len)
        if instr_len_m_diff:
            if mode == "m8" or mode == "acc_m8" or mode == "mem_m8":
                instr_len_str = str(instr_len - 1)
        elif instr_len_x_diff:
            if mode == "x8":
                instr_len_str = str(instr_len - 1)

        case_str += f"    self.pc += {instr_len_str};\n"
        case_str += "}\n"

        if "8" in mode or mode == "e" or mode == "all":
            case_str = case_str.replace("[[SIZE]]", "8")
        elif "16" in mode:
            case_str = case_str.replace("[[SIZE]]", "16")
    
    case_str = case_str.replace("[[OPCODE]]", opcode)

    # print(case_str)
    f.write(case_str + "\n")

f.close()