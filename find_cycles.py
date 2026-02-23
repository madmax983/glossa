import os
import re

def get_imports(file_path):
    imports = set()
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line.startswith('use crate::'):
                    # Basic parsing: use crate::module::...
                    parts = line.split('::')
                    if len(parts) > 1:
                        # Extract the module name immediately after `crate::`
                        module = parts[1].split(';')[0].split('{')[0].strip()
                        imports.add(module)
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    return imports

def find_cycles():
    graph = {}

    # Map file paths to module names
    for root, _, files in os.walk('src'):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                rel_path = os.path.relpath(file_path, 'src')

                if rel_path == 'lib.rs' or rel_path == 'main.rs':
                    module_name = 'root'
                else:
                    # simplistic mapping: src/foo/bar.rs -> foo
                    # src/foo.rs -> foo
                    parts = rel_path.split(os.sep)
                    module_name = parts[0].replace('.rs', '')

                if module_name not in graph:
                    graph[module_name] = set()

                deps = get_imports(file_path)
                for dep in deps:
                    if dep != module_name:
                        graph[module_name].add(dep)

    print("Dependency Graph:")
    for node, neighbors in graph.items():
        if neighbors:
            print(f"{node} -> {', '.join(neighbors)}")

    # Cycle detection
    visited = set()
    recursion_stack = set()
    cycles = []

    def dfs(node, path):
        visited.add(node)
        recursion_stack.add(node)
        path.append(node)

        if node in graph:
            for neighbor in graph[node]:
                if neighbor not in visited:
                    dfs(neighbor, path)
                elif neighbor in recursion_stack:
                    cycle_start = path.index(neighbor)
                    cycles.append(path[cycle_start:] + [neighbor])

        recursion_stack.remove(node)
        path.pop()

    for node in list(graph.keys()):
        if node not in visited:
            dfs(node, [])

    if cycles:
        print("\nCycles found:")
        for cycle in cycles:
            print(" -> ".join(cycle))
    else:
        print("\nNo cycles found at top-level module granularity.")

if __name__ == "__main__":
    find_cycles()
