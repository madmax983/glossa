
import os
import re
import sys
from collections import defaultdict

def get_modules(root_dir):
    modules = {}
    for dirpath, dirnames, filenames in os.walk(root_dir):
        for filename in filenames:
            if filename.endswith('.rs'):
                filepath = os.path.join(dirpath, filename)
                # content = open(filepath, 'r').read()

                # Derive module path from file path
                rel_path = os.path.relpath(filepath, root_dir)
                module_path = rel_path.replace('.rs', '').replace('/', '::')
                if module_path.endswith('::mod'):
                    module_path = module_path[:-5]
                elif module_path == 'lib' or module_path == 'main':
                    module_path = 'crate'

                modules[filepath] = module_path
    return modules

def get_imports(filepath):
    imports = set()
    with open(filepath, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('use crate::'):
                # Extract module path
                match = re.search(r'use crate::([\w:]+)', line)
                if match:
                    imported_mod = match.group(1).split('::')[0] # Get top-level mod
                    imports.add(imported_mod)
            elif line.startswith('use super::'):
                 imports.add('SUPER') # Placeholder for relative imports
    return imports

def build_graph(root_dir):
    modules = get_modules(root_dir)
    graph = defaultdict(set)

    # Map module name to file path for reverse lookup if needed
    mod_to_file = {v: k for k, v in modules.items()}

    for filepath, mod_name in modules.items():
        if mod_name == 'crate': continue

        # We focus on top-level modules for now to avoid noise
        top_level_mod = mod_name.split('::')[0]

        current_imports = get_imports(filepath)
        for imp in current_imports:
            if imp != 'SUPER' and imp != top_level_mod:
                 graph[top_level_mod].add(imp)

    return graph

def find_cycles(graph):
    visited = set()
    stack = set()
    cycles = []

    def dfs(node, path):
        visited.add(node)
        stack.add(node)

        if node in graph:
            for neighbor in graph[node]:
                if neighbor not in visited:
                    dfs(neighbor, path + [neighbor])
                elif neighbor in stack:
                    cycles.append(path + [neighbor])

        stack.remove(node)

    for node in graph:
        if node not in visited:
            dfs(node, [node])

    return cycles

if __name__ == "__main__":
    root = 'src'
    graph = build_graph(root)
    cycles = find_cycles(graph)

    if cycles:
        print("Found circular dependencies:")
        for cycle in cycles:
            print(" -> ".join(cycle))
    else:
        print("No top-level circular dependencies found.")
