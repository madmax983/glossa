import sys

def get_lines(filename, start, end):
    with open(filename, 'r') as f:
        lines = f.readlines()

    start_idx = max(0, start - 1)
    end_idx = min(len(lines), end)

    for i in range(start_idx, end_idx):
        print(f"{i + 1}:{lines[i]}", end="")

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python get_lines.py <filename> <start> <end>")
        sys.exit(1)

    get_lines(sys.argv[1], int(sys.argv[2]), int(sys.argv[3]))
