import os

for filename in os.listdir("tests/lemons/CPUTest/"):
    if ".sfc" in filename:
        f = open(f"tests/lemons/CPUTest/{filename}", "rb")
        data = f.read()
        f.close()

        header_start = data.find(b"65816")

        sub_checksum_hi = data[header_start + 0x1F]
        sub_checksum_lo = data[header_start + 0x1E]
        sub_complement_hi = data[header_start + 0x1D]
        sub_complement_lo = data[header_start + 0x1C]

        total_extra = sub_checksum_hi + sub_checksum_lo + sub_complement_hi + sub_complement_lo

        actual_checksum = (sum(data) - total_extra + 0x1FE) & 0xFFFF
        actual_complement = 0xFFFF - actual_checksum

        checksum_hi = actual_checksum >> 8 
        checksum_lo = actual_checksum & 0xFF
        complement_hi = actual_complement >> 8
        complement_lo = actual_complement & 0xFF

        new_header_bytes = bytes([complement_lo, complement_hi, checksum_lo, checksum_hi])

        f = open(f"tests/lemons/CPUTest/{filename}", "wb")
        new_data = data[:header_start + 0x1C] + new_header_bytes + data[header_start + 0x20:]
        f.write(new_data)
        f.close()

        print(f"'{filename}' checksum: 0x{actual_checksum:x} complement: 0x{actual_complement:x}")