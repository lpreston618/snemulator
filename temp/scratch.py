"""
IMMEDIATE - immediate
ABSOLUTE - absolute
ABSOLUTE,X - absolute_x
ABSOLUTE,Y - absolute_y
(ABSOLUTE) - absolute_indirect
(ABSOLUTE,X) - absolute_x_indirect
DIRECT - direct
DIRECT,X - direct_x
DIRECT,Y - direct_y
(DIRECT) - direct_indirect
[DIRECT] - direct_indirect_long
(DIRECT,X) - direct_x_indirect
(DIRECT),Y - direct_indirect_y
[DIRECT],Y _direct_indirect_long_y
LONG - long
LONG,X - long_x
[ABSOLUTE] - long_indirect
RELATIVE8 - relative
RELATIVE16 - relative_long
SOURCE,DESTINATION - source & destination
STACK,S - stack_relative
(STACK,S),Y - stack_relative_indirect_y

"""

addr_mode_map = {
    "imm": "Immediate",
    "imp": "Implied",
    "acc": "Accumulator",
    "abs": "Absolute",
    "abs,x": "AbsoluteX",
    "abs,y": "AbsoluteY",
    "(abs)": "AbsoluteIndirect",
    "(abs,x)": "AbsoluteXIndirect",
    "dir": "Direct",
    "dir,x": "DirectX",
    "dir,y": "DirectY",
    "(dir)": "DirectIndirect",
    "[dir]": "DirectIndirectLong",
    "(dir,x)": "DirectXIndirect",
    "(dir),y": "DirectIndirectY",
    "[dir],y": "DirectIndirectLongY",
    "long": "Long",
    "long,x": "LongX",
    "[abs]": "LongIndirect",
    "rel8": "Relative8",
    "rel16": "Relative16",
    "src,dest": "SrcDst",
    "stk,s": "StackRelative",
    "(stk,s),y": "StackRelativeIndirectY",
    
}

lines = []
with open("temp2.txt", "r") as f:
    lines = f.readlines()

c = 0
result = []
for line in lines:    
    line = line.strip()
    split = line.split()
    
    if int(split[0], 16) != c:
        print(f"Opcode {c:02X} missing")
        result.append("")
        c += 2
        continue
        
    c += 1
    
    name = split[6].lower()
    addr_mod = addr_mode_map[split[3].lower()]

    result.append("    DisassembleData {" + f'name: "{name}", addr_mode: AddressingMode::{addr_mod}' + "},")

with open("table.txt", "w") as f:
    f.write("\n".join(result))