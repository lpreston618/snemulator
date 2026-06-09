import subprocess
import os

# The base directory to start searching from
root_folder = "testroms/"
roms_type = "hirom"

for dirpath, _, filenames in os.walk(root_folder):
    for f in filenames:
        fpath = os.path.join(dirpath, f)

        if f.endswith("_fixed.smc") or f.endswith("_fixed.sfc"):
            try:
                os.remove(fpath)
            except OSError as e:
                print(f"Error deleting {fpath}: {e}")
            continue

        if f.endswith(".smc") or f.endswith(".sfc"):
            print(f"Processing: {fpath}")
            subprocess.run(["uv", "run", "./tools/fixchecksum.py", fpath, roms_type])
            print()