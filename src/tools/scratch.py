# import subprocess
# import os

# folder = "tests/blarggs/timers"

# test_list = os.listdir(folder)

# for f in test_list:
#     fpath = folder + "/" + f

#     if f.endswith("_fixed.smc") or f.endswith("_fixed.sfc"):
#         os.remove(fpath)
#         continue

#     subprocess.run(["python3", "src/tools/fixchecksum.py", fpath, "hirom"])

#     print()

lines = []
with open("aramprg.txt", "r") as f:
    lines = f.readlines()

outbytes = []
for line in lines:
    for char in line.split():
        b = int(char, base=16)
        outbytes.append(b)

outbytes = bytes(outbytes)

with open("aramprg.bin", "wb") as outf:
    outf.write(outbytes)

print("Done.")