f = open("games/cputest.sfc", "rb")

total = 0
for byte in f.read():
    total += byte

print(f"total: 0x{total:02X}")