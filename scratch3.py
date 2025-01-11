f = open("scratch.txt", "r")
opcode_lines = f.readlines()
f.close()

f = open("match.txt", "r")
match_lines = f.read()
f.close()

int_to_cycle_str_map = {
    "0": "0",
    "1": "Cpu65c816::ONE_CYCLE",
    "2": "Cpu65c816::TWO_CYCLE",
    "3": "Cpu65c816::THREE_CYCLE",
    "4": "Cpu65c816::FOUR_CYCLE",
}

xxp_template = """
                if (self.idx_size() == RegSize::Byte) && !self.page_crossed {
                    extra_clocks = [[EXTRA_CYCLES_M1]];
                } else {
                    extra_clocks = [[EXTRA_CYCLES]];
                }"""

for line in opcode_lines:
    split = line.split()

    opcode = split[0]
    extra_cycle_count = split[4]
    extra_cycle_str = int_to_cycle_str_map[extra_cycle_count]
    mode = split[5]
    

    # if mode == "ALL":
    #     last_pos = -1
    #     pos = match_lines.find(f"({opcode}", last_pos+1)
    #     while pos != -1:
    #         last_pos = pos
    #         match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)
    #         pos = match_lines.find(f"({opcode}", last_pos+1)
    # elif mode == "EMULATION":
    #     pos = match_lines.find(f"({opcode}, CpuMode::Emulation")
    #     match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)
    # elif mode == "NATIVE":
    #     pos = match_lines.find(f"({opcode}, CpuMode::Native")
    #     match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)
    # elif mode == "M_BYTE":
    #     pos = match_lines.find(f"({opcode}, _, RegSize::Byte")
    #     match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)
    # elif mode == "M_TWOBYTE":
    #     pos = match_lines.find(f"({opcode}, _, RegSize::TwoBytes")
    #     match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)
    # elif mode == "X_BYTE":
    #     pos = match_lines.find(f"({opcode}, _, _, RegSize::Byte")
    #     match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)
    # elif mode == "X_TWOBYTE":
    #     pos = match_lines.find(f"({opcode}, _, _, RegSize::TwoBytes")
    #     match_lines = match_lines[:pos] + match_lines[pos:].replace("[[EXTRA_CYCLES_HERE]]", extra_cycle_str, 1)

    if "-x+x*p" == split[-1]:
        if mode == "M_BYTE":
            pos = match_lines.find(f"({opcode}, _, RegSize::Byte")
            old_str = f"extra_clocks = {extra_cycle_str};"
            extra_cycle_m1_str = int_to_cycle_str_map[str(int(extra_cycle_count) - 1)]
            new_str = xxp_template.replace("[[EXTRA_CYCLES_M1]]", extra_cycle_m1_str)
            new_str = new_str.replace("[[EXTRA_CYCLES]]", extra_cycle_str)
            match_lines = match_lines[:pos] + match_lines[pos:].replace(old_str, new_str, 1)
        elif mode == "M_TWOBYTE":
            pos = match_lines.find(f"({opcode}, _, RegSize::TwoBytes")
            old_str = f"extra_clocks = {extra_cycle_str};"
            extra_cycle_m1_str = int_to_cycle_str_map[str(int(extra_cycle_count) - 1)]
            new_str = xxp_template.replace("[[EXTRA_CYCLES_M1]]", extra_cycle_m1_str)
            new_str = new_str.replace("[[EXTRA_CYCLES]]", extra_cycle_str)
            match_lines = match_lines[:pos] + match_lines[pos:].replace(old_str, new_str, 1)

f = open("scratch2.txt", "w")
f.write(match_lines)
f.close()