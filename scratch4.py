f = open("opcodes3.txt", "r")
opcode_lines = f.readlines()
f.close()

f = open("io_timings.txt", "r")
io_lines = f.readlines()[1:]
f.close()

timing_map = {}

for line in io_lines:
    split = line.split()
    timing_map[split[0]] = split[1:]

mode_to_idx_map = {
    "EMULATION": 0,
    "NATIVE": 1,
    "M_BYTE": 1,
    "X_BYTE": 1,
    "M_TWOBYTE": 4,
    "X_TWOBYTE": 4,
    "ALL": 4,
}

for line in opcode_lines:
    line = line.removesuffix("\n")
    
    split = line.split()
    
    opcode = split[0]
    base_cycles = int(line[23])
    mode = split[5]
    addr_mode = split[2]

    # The numbers in timings were calculated with DL == 0, so dir modes took an extra cycle.
    # This is to offset the base cycles to account for that.
    if "dir" in addr_mode:
        base_cycles += 1

    idx = mode_to_idx_map[mode]

    used_cycles = int(timing_map[opcode][idx])

    new_cycles = base_cycles - used_cycles

    print(line[:23] + str(new_cycles) + line[24:])