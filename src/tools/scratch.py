lines = []
with open("aram_disassembly.txt", "r") as f:
    lines = f.readlines()

c = 0
for line in lines:
    if line.strip() == "":
        continue

    c += 1

    print(f"0x{line[1:5]}, ", end="")

    if c % 16 == 0:
        print()