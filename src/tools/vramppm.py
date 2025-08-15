import sys, os

if __name__ == "__main__":
    vram_filepath = input("Input VRAM Dump (.bin) file: ")

    if not os.path.exists(vram_filepath):
        print(f"file '{vram_filepath}' not found, exiting.")
        sys.exit(0)
    if not vram_filepath.endswith(".bin"):
        print(f"file '{vram_filepath}' not a .bin file, exiting.")
        sys.exit(0)
    
    vram_data = None
    with open(vram_filepath, "rb") as f:
        vram_data = f.read()
    
    start_addr_str = input("Enter the chr start address (in hex): 0x")
    start_addr = 0
    try:
        start_addr = int(start_addr_str, 16)
        if start_addr > 0xFFFF or start_addr < 0:
            raise ValueError()
    except:
        print(f"Value '{start_addr_str}' not a valid start address, exiting.")
        sys.exit(0)
    
    bpp_str = input("How many bits per pixel (2, 4, 8)? ")
    bpp = 0
    try:
        bpp = int(bpp_str)
        if bpp != 2 and bpp != 4 and bpp != 8:
            raise ValueError()
    except:
        print(f"Value '{bpp_str}' not a valid color depth, exiting.")
        sys.exit(0)
    
    nColors = 1 << bpp
    color_palette = [i * (256//nColors) for i in range(nColors)]

    chrLen = bpp * 4
    nChrs = 32*32

    vram_word_data = []

    for i in range(start_addr, start_addr+(nChrs*chrLen*2)):
        idx = (i*2) & 0xFFFF

        vram_word = vram_data[idx + 0]
        vram_word |= vram_data[idx + 1] << 8

        vram_word_data.append(vram_word)

    ppm_str = "P3 256 256\n255\n"

    for y in range(256):
        chrY = y // 8
        chrRow = y % 8

        for x in range(256):
            chrX = x // 8
            chrCol = x % 8
            chrIdx = chrY * 32 + chrX

            if bpp == 2:
                bp01 = vram_word_data[chrIdx*chrLen + chrRow]
                b0 = (bp01 >> (7-chrCol)) & 1
                b1 = (bp01 >> (15-chrCol)) & 1
                pal_idx = (b1 << 1) | b0
                col = color_palette[pal_idx]
                ppm_str += f"{col} {col} {col}\n"
            elif bpp == 4:
                bp01 = vram_word_data[chrIdx*chrLen + chrRow + 0]
                bp23 = vram_word_data[chrIdx*chrLen + chrRow + 1]
                b0 = (bp01 >> (7-chrCol)) & 1
                b1 = (bp01 >> (15-chrCol)) & 1
                b2 = (bp23 >> (7-chrCol)) & 1
                b3 = (bp23 >> (15-chrCol)) & 1
                pal_idx = (b3 << 3) | (b2 << 2) | (b1 << 1) | b0
                col = color_palette[pal_idx]
                ppm_str += f"{col} {col} {col}\n"

    with open("vramchr.ppm", "w") as f:
        f.write(ppm_str)

    print("Done.")