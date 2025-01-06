f = open("scratch.txt", "r")

instr_lines = []

for line in f.readlines():
    if len(line.strip()) == 0:
        continue

    instr_lines.append(line.strip())

instr_lines.sort()

f.close()


f = open("opcodes.txt", "w")

for line in instr_lines:
    split = line.split()

    opcode = split[0]
    instr_len: str = split[1]
    cycles = split[2]
    addr_mode: str = split[3]
    name = split[6]

    if name == "PEA" or name == "PEI" or name == "PER":
        name = "PEX"
    
    if name == "BRL":
        name = "BRA"

    f.write(f"0x{opcode} {name} {addr_mode.ljust(9)} {instr_len.ljust(3)} {cycles}\n")
    # print(f"0x{opcode} {name} {addr_mode}")

f.close()