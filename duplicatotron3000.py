args_diffs = {
    "m8": "cpu: &mut Cpu65c816, address: u32",
    "m16": "cpu: &mut Cpu65c816, address_lo: u32, address_hi: u32",
    "x8": "cpu: &mut Cpu65c816, address: u32",
    "x16": "cpu: &mut Cpu65c816, address_lo: u32, address_hi: u32",
    "n8": "cpu: &mut Cpu65c816, address: u32",
    "n16": "cpu: &mut Cpu65c816, address: u32",
    "e": "cpu: &mut Cpu65c816, address: u32",
}
version_diffs = {
    "read(address)": {
        "m8": "read8(address)",
        "m16": "read16(address_lo, address_hi)",
        "x8": "read8(address)",
        "x16": "read16(address_lo, address_hi)",
        "n8": "read8(address)",
        "n16": "read16(address)",
        "e": "read8(address)",
    },
    "write(address)": {
        "m8": "write8(address, data)",
        "m16": "write16(address_lo, address_hi, data)",
        "x8": "write8(address, data)",
        "x16": "write16(address_lo, address_hi, data)",
        "n8": "write8(address, data)",
        "n16": "write16(address_lo, address_hi, data)",
        "e": "write8(address, data)",
    },
    "get_acc": {
        "m8": "get_acc_lo",
        "m16": "get_acc",
        "n8": "get_acc_lo",
        "n16": "get_acc",
        "e": "get_acc_lo",
    },
    "set_acc": {
        "m8": "set_acc_lo",
        "m16": "set_acc",
        "n8": "set_acc_lo",
        "n16": "set_acc",
        "e": "set_acc_lo",
    },
    "get_x": {
        "x8": "get_x_lo",
        "x16": "get_x",
        "n8": "get_x_lo",
        "n16": "get_x",
        "e": "get_x_lo",
    },
    "set_x": {
        "x8": "set_x_lo",
        "x16": "set_x",
        "n8": "set_x_lo",
        "n16": "set_x",
        "e": "set_x_lo",
    },
    "get_y": {
        "x8": "get_y_lo",
        "x16": "get_y",
        "n8": "get_y_lo",
        "n16": "get_y",
        "e": "get_y_lo",
    },
    "set_y": {
        "x8": "set_y_lo",
        "x16": "set_y",
        "n8": "set_y_lo",
        "n16": "set_y",
        "e": "set_y_lo",
    },
}

def generate_versions(funcName, funcBody, versionsList):
    result = ""

    for version in versionsList:
        header = f"fn {funcName}_{version}({args_diffs[version]}) " + "{\n"

        versionedBody = funcBody
        
        for dummy_func in version_diffs.keys():
            if replacement := version_diffs[dummy_func].get(version):
                print(f"Replacing '{dummy_func}' with {version} version '{replacement}'")
                versionedBody = versionedBody.replace(dummy_func, replacement)
        
        result += header

        for line in versionedBody.split("\n"):
            result += f"    {line}\n"
        
        result += "}\n\n"
    
    return result


def main():
    name = "bit"
    body = """let result = accReg & read(address);

set_flag_n!(result, u8);
set_flag_z!(result);
"""
    versions = ["m8", "m16"]

    result = generate_versions(name, body, versions)

    f = open("generated_func.rs", "w")
    f.write(result)
    f.close()

    print("Finished :)")


if __name__ == "__main__":
    main()
