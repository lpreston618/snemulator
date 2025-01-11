f = open("opcodes.txt", "r")
opcode_lines = f.readlines()
f.close()

f = open("io_timings.txt", "r")
timing_lines = f.readlines()
f.close()

f = open("opcodes2.txt", "r")
opcodes_new_lines = f.readlines()
f.close()

base_cycles = [int(line[23]) for line in opcode_lines]
io_ops = [int(line[28]) for line in timing_lines]

new_cycles = []

for i in range(256):
    new_cycles.append(base_cycles[i] - io_ops[i])


i = 0
for j, line in enumerate(opcodes_new_lines):
    if line[:2] == "0x":
        opcodes_new_lines[j] = line[:23] + str(new_cycles[i]) + line[24:]
        i += 1

    # opcodes_new_lines[j] = opcodes_new_lines[j].removesuffix("\n")
    
    print(opcodes_new_lines[j].removesuffix("\n"))


with open("opcodes3.txt", "w") as f:
    f.writelines(opcodes_new_lines)