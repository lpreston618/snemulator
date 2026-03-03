b0, b1 = None, None
with open("testroms/gilyon/spctest/spc_tests0.spc", "rb") as f0:
    with open("testroms/gilyon/spctest/spc_tests1.spc", "rb") as f1:
# with open("aram_dump1.bin", "rb") as f0:
#     with open("aram_dump0.bin", "rb") as f1:
    # with open("testroms/gilyon/spctest/spc_tests1.spc", "rb") as f1:
        b0 = f0.read()
        b1 = f1.read()

n = min(len(b0), len(b1))

print(f"Comparing 0x{n - 0x300:X} bytes")

for i in range(0x300, n):
    if b0[i] != b1[i]:
        print(f"Discrepency at ${i:04X}")