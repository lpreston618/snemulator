lines = []
with open("temp3.txt", "r") as f:
    lines = f.readlines()

lines.sort(key=lambda line: line[:2])

with open("temp2.txt", "w") as f:
    f.write("".join(lines))
