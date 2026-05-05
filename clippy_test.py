import subprocess
import os

def check_clippy():
    result = subprocess.run(["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"], capture_output=True, text=True)
    if result.returncode == 0:
        print("Clippy check passed successfully.")
    else:
        print("Clippy errors found:")
        print(result.stderr)

if __name__ == "__main__":
    check_clippy()
