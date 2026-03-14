import sys

def get_lines(filename, start_line, num_lines):
    with open(filename, 'r') as f:
        lines = f.readlines()

    end_line = min(start_line + num_lines, len(lines))

    for i in range(start_line - 1, end_line - 1):
        print(f"{i+1}: {lines[i]}", end='')

if __name__ == "__main__":
    get_lines(sys.argv[1], int(sys.argv[2]), int(sys.argv[3]))
