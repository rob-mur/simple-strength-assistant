import sys
with open(sys.argv[1], 'r') as f:
    lines = f.readlines()
with open(sys.argv[1], 'w') as f:
    for line in lines:
        if line.startswith('pick ') and any(x in line for x in ['Visual polish', 'Correct Step', 'Stabilize Tape', 'Finalize Tape']):
            f.write(line.replace('pick ', 'reword ', 1))
        else:
            f.write(line)
