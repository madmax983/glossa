import subprocess
print(subprocess.check_output(['git', 'diff', '--no-prefix']).decode('utf-8'))
