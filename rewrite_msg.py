import sys
with open(sys.argv[1], 'r') as f:
    lines = f.readlines()
if ': ' in lines[0]:
    parts = lines[0].split(': ', 1)
    if len(parts) > 1 and parts[1]:
        lines[0] = parts[0] + ': ' + parts[1][0].lower() + parts[1][1:]
with open(sys.argv[1], 'w') as f:
    f.writelines(lines)
