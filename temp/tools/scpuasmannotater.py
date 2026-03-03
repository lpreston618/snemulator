import sys

def is_instr_line(l):
    if len(l.strip()) == 0:
        return False
    first = l.split()[0]
    return len(first) == 8 and first[0] == "$"

def get_target_addr(l):
    last = l.split()[-1]
    
    if len(last) == 5 and last[0] == "$":
        try:
            return int(l.split()[-1][1:], 16)
        except:
            return None


named_addrs = {
    0x2100: "F... BBBB   Forced blanking (F), screen brightness (B)",
    0x2101: "SSSN NbBB   OBJ sprite size (S), name secondary select (N), name base address (B)",
    0x2102: "AAAA AAAA   OAM word address (low byte)",
    0x2103: "P... ...B   Priority rotation (P), address high bit (B), OAM word address (high byte)",
    0x2104: "DDDD DDDD   OAM data write byte (2x for word), increments OAMADD byte",
    0x2105: "4321 PMMM   Tilemap tile size (#), BG3 priority (P), BG mode (M)",
    0x2106: "SSSS 4321   Mosaic size (S), mosaic BG enable (#)",
    0x2107: "AAAA AAYX   Tilemap VRAM address (A), vertical/horizontal tilemap count (Y/X)",
    0x2108: "AAAA AAYX   Tilemap VRAM address (A), vertical/horizontal tilemap count (Y/X)",
    0x2109: "AAAA AAYX   Tilemap VRAM address (A), vertical/horizontal tilemap count (Y/X)",
    0x210A: "AAAA AAYX   Tilemap VRAM address (A), vertical/horizontal tilemap count (Y/X)",
    0x210B: "BBBB AAAA   BG2 CHR base address (B), BG1 CHR base address (A)",
    0x210C: "DDDD CCCC   BG4 CHR base address (D), BG3 CHR base address (C)",
    0x210D: ".... ..XX XXXX XXXX   BG1 horizontal scroll (X)",
    0x210E: ".... ..YY YYYY YYYY   BG1 vertical scroll (Y)",
    0x210F: ".... ..XX XXXX XXXX   BG2 horizontal scroll (X)",
    0x2110: ".... ..YY YYYY YYYY   BG2 vertical scroll (Y)",
    0x2111: ".... ..XX XXXX XXXX   BG3 horizontal scroll (X)",
    0x2112: ".... ..YY YYYY YYYY   BG3 vertical scroll (Y)",
    0x2113: ".... ..XX XXXX XXXX   BG4 horizontal scroll (X)",
    0x2114: ".... ..YY YYYY YYYY   BG4 vertical scroll (Y)",
    0x2115: "M... RRII   VRAM address increment mode (M), remapping (R), increment size (I)",
    0x2116: "LLLL LLLL   VRAM word address (low byte)",
    0x2117: "hHHH HHHH   VRAM word address (high byte)",
    0x2118: "LLLL LLLL   VRAM data write (low byte)",
    0x2119: "HHHH HHHH   VRAM data write (high byte)",
    0x211A: "RF.. ..YX   Mode 7 tilemap repeat (R), fill (F), flip vertical (Y), flip horizontal (X)",
    0x211B: "DDDD DDDD dddd dddd   Mode 7 matrix A or signed 16-bit multiplication factor",
    0x211C: "DDDD DDDD dddd dddd   Mode 7 matrix B or signed 8-bit multiplication factor",
    0x211D: "DDDD DDDD dddd dddd   Mode 7 matrix C",
    0x211E: "DDDD DDDD dddd dddd   Mode 7 matrix D",
    0x211F: "...X XXXX XXXX XXXX   Mode 7 center X",
    0x2120: "...Y YYYY YYYY YYYY   Mode 7 center Y",
    0x2121: "AAAA AAAA   CGRAM word address",
    0x2122: ".BBB BBGG GGGR RRRR   CGRAM data write, increments CGADD byte address after each write",
    0x2123: "DdCc BbAa   Enable (ABCD) and Invert (abcd) windows for BG1 and BG2",
    0x2124: "DdCc BbAa   Enable (EFGH) and Invert (efgh) windows for BG3 and BG4",
    0x2125: "LlKk JjIi   Enable (IJKL) and Invert (ijkl) windows for OBJ and color",
    0x2126: "LLLL LLLL   Window 1 left position",
    0x2127: "RRRR RRRR   Window 1 right position",
    0x2128: "LLLL LLLL   Window 2 left position",
    0x2129: "RRRR RRRR   Window 2 right position",
    0x212A: "4433 2211   Window mask logic for BG layers (00=OR, 01=AND, 10=XOR, 11=XNOR)",
    0x212B: ".... CCOO   Window mask logic for OBJ and color",
    0x212C: "...O 4321   Main screen layer enable",
    0x212D: "...O 4321   Sub screen layer enable",
    0x212E: "...O 4321   Main screen layer window enable",
    0x212F: "...O 4321   Sub screen layer window enable",
    0x2130: "MMSS ..AD   Main/sub screen color window black/transparent regions (MS), fixed/subscreen (A), direct color (D)",
    0x2131: "MHBO 4321   Color math add/subtract (M), half (H), backdrop (B), layer enable (O4321)",
    0x2132: "BGRC CCCC   Fixed color channel select (BGR) and value (C)",
    0x2133: "EX.. HOiI   External sync (E), EXTBG (X), Hi-res (H), Overscan (O), OBJ interlace (i), Screen interlace (I)",
    0x2134: "LLLL LLLL   24-bit signed multiplication result (low byte)",
    0x2135: "MMMM MMMM   24-bit signed multiplication result (middle byte)",
    0x2136: "HHHH HHHH   24-bit signed multiplication result (high byte)",
    0x2137: ".... ....   Software latch for H/V counters",
    0x2138: "DDDD DDDD   Read OAM data byte, increments OAMADD byte",
    0x2139: "LLLL LLLL   VRAM data read (low byte), increments VMADD",
    0x213A: "HHHH HHHH   VRAM data read (high byte), increments VMADD",
    0x213B: ".BBB BBGG GGGR RRRR   CGRAM data read, increments CGADD",
    0x213C: "...H HHHH HHHH HHHH   Output horizontal counter",
    0x213D: "...V VVVV VVVV VVVV   Output vertical counter",
    0x213E: "TRM. VVVV   Sprite overflow (T), sprite tile overflow (R), master/slave (M), PPU1 version (V)",
    0x213F: "FL.M VVVV   Interlace field (F), counter latch value (L), NTSC/PAL (M), PPU2 version (V)",

    0x2140: "APUIO 0",
    0x2141: "APUIO 1",
    0x2142: "APUIO 2",
    0x2143: "APUIO 3",

    0x4200: "N.VH ...J  Vblank NMI enable (N), timer IRQ mode (VH), joypad auto-read enable (J)",

    0x420B: "MDMAEN: 7654 3210   DMA enable",
    0x420C: "HDMAEN: 7654 3210   HDMA enable",
    0x4300: "DI.A APPP   Ch. 0: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4310: "DI.A APPP   Ch. 1: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4320: "DI.A APPP   Ch. 2: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4330: "DI.A APPP   Ch. 3: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4340: "DI.A APPP   Ch. 4: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4350: "DI.A APPP   Ch. 5: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4360: "DI.A APPP   Ch. 6: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4370: "DI.A APPP   Ch. 7: Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
    0x4301: "AAAA AAAA   Ch. 0: B-bus address",
    0x4311: "AAAA AAAA   Ch. 1: B-bus address",
    0x4321: "AAAA AAAA   Ch. 2: B-bus address",
    0x4331: "AAAA AAAA   Ch. 3: B-bus address",
    0x4341: "AAAA AAAA   Ch. 4: B-bus address",
    0x4351: "AAAA AAAA   Ch. 5: B-bus address",
    0x4361: "AAAA AAAA   Ch. 6: B-bus address",
    0x4371: "AAAA AAAA   Ch. 7: B-bus address",
    0x4302: "LLLL LLLL   Ch. 0: DMA source address low byte / HDMA table start address low",
    0x4312: "LLLL LLLL   Ch. 1: DMA source address low byte / HDMA table start address low",
    0x4322: "LLLL LLLL   Ch. 2: DMA source address low byte / HDMA table start address low",
    0x4332: "LLLL LLLL   Ch. 3: DMA source address low byte / HDMA table start address low",
    0x4342: "LLLL LLLL   Ch. 4: DMA source address low byte / HDMA table start address low",
    0x4352: "LLLL LLLL   Ch. 5: DMA source address low byte / HDMA table start address low",
    0x4362: "LLLL LLLL   Ch. 6: DMA source address low byte / HDMA table start address low",
    0x4372: "LLLL LLLL   Ch. 7: DMA source address low byte / HDMA table start address low",
    0x4303: "HHHH HHHH   Ch. 0: DMA source address high byte / HDMA table start address high",
    0x4313: "HHHH HHHH   Ch. 1: DMA source address high byte / HDMA table start address high",
    0x4323: "HHHH HHHH   Ch. 2: DMA source address high byte / HDMA table start address high",
    0x4333: "HHHH HHHH   Ch. 3: DMA source address high byte / HDMA table start address high",
    0x4343: "HHHH HHHH   Ch. 4: DMA source address high byte / HDMA table start address high",
    0x4353: "HHHH HHHH   Ch. 5: DMA source address high byte / HDMA table start address high",
    0x4363: "HHHH HHHH   Ch. 6: DMA source address high byte / HDMA table start address high",
    0x4373: "HHHH HHHH   Ch. 7: DMA source address high byte / HDMA table start address high",
    0x4304: "BBBB BBBB   Ch. 0: DMA source address bank byte / HDMA table start bank",
    0x4314: "BBBB BBBB   Ch. 1: DMA source address bank byte / HDMA table start bank",
    0x4324: "BBBB BBBB   Ch. 2: DMA source address bank byte / HDMA table start bank",
    0x4334: "BBBB BBBB   Ch. 3: DMA source address bank byte / HDMA table start bank",
    0x4344: "BBBB BBBB   Ch. 4: DMA source address bank byte / HDMA table start bank",
    0x4354: "BBBB BBBB   Ch. 5: DMA source address bank byte / HDMA table start bank",
    0x4364: "BBBB BBBB   Ch. 6: DMA source address bank byte / HDMA table start bank",
    0x4374: "BBBB BBBB   Ch. 7: DMA source address bank byte / HDMA table start bank",
    0x4305: "LLLL LLLL   Ch. 0: DMA byte count low / HDMA indirect address low",
    0x4315: "LLLL LLLL   Ch. 1: DMA byte count low / HDMA indirect address low",
    0x4325: "LLLL LLLL   Ch. 2: DMA byte count low / HDMA indirect address low",
    0x4335: "LLLL LLLL   Ch. 3: DMA byte count low / HDMA indirect address low",
    0x4345: "LLLL LLLL   Ch. 4: DMA byte count low / HDMA indirect address low",
    0x4355: "LLLL LLLL   Ch. 5: DMA byte count low / HDMA indirect address low",
    0x4365: "LLLL LLLL   Ch. 6: DMA byte count low / HDMA indirect address low",
    0x4375: "LLLL LLLL   Ch. 7: DMA byte count low / HDMA indirect address low",
    0x4306: "HHHH HHHH   Ch. 0: DMA byte count high / HDMA indirect address high",
    0x4316: "HHHH HHHH   Ch. 1: DMA byte count high / HDMA indirect address high",
    0x4326: "HHHH HHHH   Ch. 2: DMA byte count high / HDMA indirect address high",
    0x4336: "HHHH HHHH   Ch. 3: DMA byte count high / HDMA indirect address high",
    0x4346: "HHHH HHHH   Ch. 4: DMA byte count high / HDMA indirect address high",
    0x4356: "HHHH HHHH   Ch. 5: DMA byte count high / HDMA indirect address high",
    0x4366: "HHHH HHHH   Ch. 6: DMA byte count high / HDMA indirect address high",
    0x4376: "HHHH HHHH   Ch. 7: DMA byte count high / HDMA indirect address high",
    0x4307: "BBBB BBBB   Ch. 0: HDMA indirect table bank",
    0x4317: "BBBB BBBB   Ch. 1: HDMA indirect table bank",
    0x4327: "BBBB BBBB   Ch. 2: HDMA indirect table bank",
    0x4337: "BBBB BBBB   Ch. 3: HDMA indirect table bank",
    0x4347: "BBBB BBBB   Ch. 4: HDMA indirect table bank",
    0x4357: "BBBB BBBB   Ch. 5: HDMA indirect table bank",
    0x4367: "BBBB BBBB   Ch. 6: HDMA indirect table bank",
    0x4377: "BBBB BBBB   Ch. 7: HDMA indirect table bank",
    0x4308: "LLLL LLLL   Ch. 0: HDMA table current address low",
    0x4318: "LLLL LLLL   Ch. 1: HDMA table current address low",
    0x4328: "LLLL LLLL   Ch. 2: HDMA table current address low",
    0x4338: "LLLL LLLL   Ch. 3: HDMA table current address low",
    0x4348: "LLLL LLLL   Ch. 4: HDMA table current address low",
    0x4358: "LLLL LLLL   Ch. 5: HDMA table current address low",
    0x4368: "LLLL LLLL   Ch. 6: HDMA table current address low",
    0x4378: "LLLL LLLL   Ch. 7: HDMA table current address low",
    0x4309: "HHHH HHHH   Ch. 0: HDMA table current address high",
    0x4319: "HHHH HHHH   Ch. 1: HDMA table current address high",
    0x4329: "HHHH HHHH   Ch. 2: HDMA table current address high",
    0x4339: "HHHH HHHH   Ch. 3: HDMA table current address high",
    0x4349: "HHHH HHHH   Ch. 4: HDMA table current address high",
    0x4359: "HHHH HHHH   Ch. 5: HDMA table current address high",
    0x4369: "HHHH HHHH   Ch. 6: HDMA table current address high",
    0x4379: "HHHH HHHH   Ch. 7: HDMA table current address high",
    0x430A: "RLLL LLLL   Ch. 0: HDMA reload flag (R) and scanline counter (L)",
    0x431A: "RLLL LLLL   Ch. 1: HDMA reload flag (R) and scanline counter (L)",
    0x432A: "RLLL LLLL   Ch. 2: HDMA reload flag (R) and scanline counter (L)",
    0x433A: "RLLL LLLL   Ch. 3: HDMA reload flag (R) and scanline counter (L)",
    0x434A: "RLLL LLLL   Ch. 4: HDMA reload flag (R) and scanline counter (L)",
    0x435A: "RLLL LLLL   Ch. 5: HDMA reload flag (R) and scanline counter (L)",
    0x436A: "RLLL LLLL   Ch. 6: HDMA reload flag (R) and scanline counter (L)",
    0x437A: "RLLL LLLL   Ch. 7: HDMA reload flag (R) and scanline counter (L)",
    0x430B: "DDDD DDDD   Ch. 0: Unused shared data byte",
    0x431B: "DDDD DDDD   Ch. 1: Unused shared data byte",
    0x432B: "DDDD DDDD   Ch. 2: Unused shared data byte",
    0x433B: "DDDD DDDD   Ch. 3: Unused shared data byte",
    0x434B: "DDDD DDDD   Ch. 4: Unused shared data byte",
    0x435B: "DDDD DDDD   Ch. 5: Unused shared data byte",
    0x436B: "DDDD DDDD   Ch. 6: Unused shared data byte",
    0x437B: "DDDD DDDD   Ch. 7: Unused shared data byte",
    0x430F: "DDDD DDDD   Ch. 0: Unused shared data byte",
    0x431F: "DDDD DDDD   Ch. 1: Unused shared data byte",
    0x432F: "DDDD DDDD   Ch. 2: Unused shared data byte",
    0x433F: "DDDD DDDD   Ch. 3: Unused shared data byte",
    0x434F: "DDDD DDDD   Ch. 4: Unused shared data byte",
    0x435F: "DDDD DDDD   Ch. 5: Unused shared data byte",
    0x436F: "DDDD DDDD   Ch. 6: Unused shared data byte",
    0x437F: "DDDD DDDD   Ch. 7: Unused shared data byte",
}

if __name__ == "__main__":
    infile = sys.argv[1]

    if not infile.endswith(".txt"):
        print("Must give .txt log file containing only scpu disassembly lines")

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

    for i in range(len(lines)):
        lines[i] = lines[i].replace("[libretro INFO] [Snemulator] ", "")

    with open(f"{infile[:-4]}_annotated.txt", "w") as outf:
        outf.writelines(lines)

# a = '''0x43n0: "DI.A APPP   Direction (D), indirect HDMA (I), address increment mode (A), transfer pattern (P)",
# 0x43n1: "AAAA AAAA   B-bus address",
# 0x43n2: "LLLL LLLL   DMA source address low byte / HDMA table start address low",
# 0x43n3: "HHHH HHHH   DMA source address high byte / HDMA table start address high",
# 0x43n4: "BBBB BBBB   DMA source address bank byte / HDMA table start bank",
# 0x43n5: "LLLL LLLL   DMA byte count low / HDMA indirect address low",
# 0x43n6: "HHHH HHHH   DMA byte count high / HDMA indirect address high",
# 0x43n7: "BBBB BBBB   HDMA indirect table bank",
# 0x43n8: "LLLL LLLL   HDMA table current address low",
# 0x43n9: "HHHH HHHH   HDMA table current address high",
# 0x43nA: "RLLL LLLL   HDMA reload flag (R) and scanline counter (L)",
# 0x43nB: "DDDD DDDD   Unused shared data byte",
# 0x43nF: "DDDD DDDD   Unused shared data byte",'''

# for line in a.split('\n'):
#     for n in range(8):
#         print(line[:4] + str(n) + line[5:21] + f"Ch. {n}: " + line[21:])