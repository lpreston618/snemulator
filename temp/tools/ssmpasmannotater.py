import sys

def is_instr_line(l):
    if len(l.strip()) == 0:
        return False
    first = l.split()[0]
    return len(first) == 6 and first[0] == "$"

def get_target_addr(l):
    last = l.split()[-1]
    
    if (len(last) == 5 or len(last) == 3) and last[0] == "$":
        try:
            return int(l.split()[-1][1:], 16)
        except:
            return None


named_addrs = {
    0x00F1: "I.CC .210  Enable IPL ROM (I), Clear data ports (C), timer enable (2,1,0).",
    0x00F2: "RAAA AAAA 	DSP register read only (R), DSP register address (A)",
    0x00F3: "DSP register data",
    
    0x00F4: "APUIO 0",
    0x00F5: "APUIO 1",
    0x00F6: "APUIO 2",
    0x00F7: "APUIO 3",

    0x00FA: "Timer 0 target.",
    0x00FB: "Timer 1 target.",
    0x00FC: "Timer 2 target.",

    0x00FD: "Timer 0 count.",
    0x00FE: "Timer 1 count.",
    0x00FF: "Timer 2 count.",
}

if __name__ == "__main__":
    infile = sys.argv[1]

    if not infile.endswith(".txt"):
        print("Must give .txt log file containing only ssmp disassembly lines")

    lines = []
    with open(infile, "r") as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if not is_instr_line(line):
            continue

        target_addr = get_target_addr(line)

        if target_addr == None:
            continue

        if named_addrs.get(target_addr, False):
            lines[i] = line[:-1] + "  ;  " + named_addrs.get(target_addr) + "\n"

    with open(f"{infile[:-4]}_annotated.txt", "w") as outf:
        outf.writelines(lines)