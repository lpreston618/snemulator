import subprocess
import os

folder = "testroms/blarggs/execio"

test_list = os.listdir(folder)

for f in test_list:
    fpath = folder + "/" + f

    if f.endswith("_fixed.smc") or f.endswith("_fixed.sfc"):
        os.remove(fpath)
        continue

    subprocess.run(["python3", "src/tools/fixchecksum.py", fpath, "hirom"])

    print()















