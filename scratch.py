f = open("opcodes3.txt", "r")
op_lines = f.readlines()
f.close()


for line in op_lines:
    line = line.removesuffix("\n")
    base_cycles = int(line[23])
    modifier = line[24:]
    end = ""

    if "-x+x*p" in modifier:
        modifier = modifier.replace("-x+x*p", "")
        end = "    -x+x*p"
    if "-2*x+x*p" in modifier:
        modifier = modifier.replace("-2*x+x*p", "")
        end = "    -2*x+x*p"

    if "-e" in modifier:
        print(line[:23] + f"{base_cycles - 1}     EMULATION" + end)
        print(line[:23] + f"{base_cycles}     NATIVE" + end)
    elif "-m" in modifier:
        print(line[:23] + f"{base_cycles - 1}     M BYTE" + end)
        print(line[:23] + f"{base_cycles}     M TWOBYTE" + end)
    elif "-2*m" in modifier:
        print(line[:23] + f"{base_cycles - 2}     M BYTE" + end)
        print(line[:23] + f"{base_cycles}     M TWOBYTE" + end)
    elif "-x" in modifier:
        print(line[:23] + f"{base_cycles - 1}     X BYTE" + end)
        print(line[:23] + f"{base_cycles}     X TWOBYTE" + end)
    else:
        print(line + "     ALL" + end)