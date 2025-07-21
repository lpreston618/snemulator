"""
Python script to compute the checksum of a ROM and store it in the header.
"""

import sys, math

CHECKSUM_OFFSET = 0x1E
COMPLEMENT_OFFSET = 0x1C

def is_pow2(n):
    if n <= 0:
        return False
    return (n & (n-1)) == 0

def count_ones(n):
    return bin(n).count("1")

def pad_rom(bytes):
    n = len(bytes)
    nOnes = count_ones(n)

    if nOnes == 1:
        return bytes

    if nOnes == 2:
        bitwidth = int(math.log2(n))
        larger_size = 1 << bitwidth
        smaller_size = n & (larger_size - 1)
        repeat_count = larger_size // smaller_size

        padded_rom = list(bytes[:larger_size])
        
        for _ in range(repeat_count):
            padded_rom.extend(list(bytes[larger_size:]))
        
        return padded_rom

    if nOnes == 3:
        bitwidth = int(math.log2(n))
        larger_size = 1 << bitwidth
        smaller_size = n & (larger_size - 1)
        smaller_pow2_size = 1 << (len(bin(n)) - 2)
        repeat_count = larger_size // smaller_pow2_size

        padded_rom = list(bytes[:larger_size])
        smaller_part = list(bytes[larger_size:])

        smaller_part.extend([0 for _ in range(smaller_pow2_size - len(smaller_part))])
        
        for _ in range(repeat_count):
            padded_rom.extend(smaller_part)
        
        return padded_rom

def fix_checksum(padded_rom: list[int], header_pos: int):
    rom_mirror = len(padded_rom) - 1

    padded_rom[(header_pos + CHECKSUM_OFFSET + 0) & rom_mirror] = 0
    padded_rom[(header_pos + CHECKSUM_OFFSET + 1) & rom_mirror] = 0
    padded_rom[(header_pos + COMPLEMENT_OFFSET + 0) & rom_mirror] = 0xFF
    padded_rom[(header_pos + COMPLEMENT_OFFSET + 1) & rom_mirror] = 0xFF
    
    checksum = sum(padded_rom) & 0xFFFF
    complement = 0xFFFF - checksum

    padded_rom[(header_pos + CHECKSUM_OFFSET + 0) & rom_mirror] = checksum & 0xFF
    padded_rom[(header_pos + CHECKSUM_OFFSET + 1) & rom_mirror] = checksum >> 8
    padded_rom[(header_pos + COMPLEMENT_OFFSET + 0) & rom_mirror] = complement & 0xFF
    padded_rom[(header_pos + COMPLEMENT_OFFSET + 1) & rom_mirror] = complement >> 8

    return padded_rom

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python fixchecks.py [ROM filepath] [Mapping mode]")
        print("    Where [ROM filepath] provides the path of a .sfc or .smc file")
        print("    and [Mapping mode] is one of 'lorom', 'hirom', and 'exhirom'")
        sys.exit()

    rom_filepath = sys.argv[1]
    rom_mapping_mode = sys.argv[2]

    print(f"Fixing checksum of ROM at '{rom_filepath}' assuming mapping mode '{rom_mapping_mode}'")

    if not rom_filepath.endswith(".sfc") and not rom_filepath.endswith(".smc"):
        print("ERROR: File must be .sfc or .smc ROM")
        sys.exit()

    rom_bytes = []
    with open(rom_filepath, "rb") as rom_file:
        rom_bytes = list(rom_file.read())

    header = []
    if len(rom_bytes) % 1024 == 512:
        header = rom_bytes[:512]
        rom_bytes = rom_bytes[512:]

    padded_rom = pad_rom(rom_bytes)

    header_pos = 0
    if rom_mapping_mode.lower() == "lorom":
        header_pos = 0x7FC0
    elif rom_mapping_mode.lower() == "hirom":
        header_pos = 0xFFC0
    elif rom_mapping_mode.lower() == "exhirom":
        header_pos = 0x40FFC0
    else:
        print(f"ERROR: mapping mode '{rom_mapping_mode}' not recognized")
        sys.exit(1)

    fixed_rom = fix_checksum(padded_rom, header_pos)[:len(rom_bytes)]

    fixed_rom_filepath = rom_filepath[:-4] + "_fixed" + rom_filepath[-4:]

    print(f"Writing fixed ROM to '{fixed_rom_filepath}'")
    
    with open(fixed_rom_filepath, "wb") as out_file:
        out_file.write(bytes(header + fixed_rom))
    
    print("Verifying ROM integrity...")

    with open(rom_filepath, "rb") as rom_file:
        with open(fixed_rom_filepath, "rb") as new_rom_file:
            old_bytes = list(rom_file.read())
            new_bytes = list(new_rom_file.read())

            if len(old_bytes) != len(new_bytes):
                print("ERROR: Fixed ROM is of different length than original.")
                sys.exit()

            old_bytes[header_pos + CHECKSUM_OFFSET + 0] = 0
            old_bytes[header_pos + CHECKSUM_OFFSET + 1] = 0
            old_bytes[header_pos + COMPLEMENT_OFFSET + 0] = 0xFF
            old_bytes[header_pos + COMPLEMENT_OFFSET + 1] = 0xFF

            new_bytes[header_pos + CHECKSUM_OFFSET + 0] = 0
            new_bytes[header_pos + CHECKSUM_OFFSET + 1] = 0
            new_bytes[header_pos + COMPLEMENT_OFFSET + 0] = 0xFF
            new_bytes[header_pos + COMPLEMENT_OFFSET + 1] = 0xFF

            for i in range(len(old_bytes)):
                if old_bytes[i] != new_bytes[i]:
                    print(f"ERROR: Fixed ROM has discrepency at ${i:06X}")
                    sys.exit()
    
    print("Done.")