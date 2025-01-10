def cycles_then_addr_mode(line):
    split = line.split()
    return split[4][1:] + split[2]

f = open("opcodes.txt", "r")
lines = f.readlines()
f.close()

lines.sort(key = cycles_then_addr_mode)

f = open("opcodes2.txt", "w")
f.writelines(lines)
f.close()