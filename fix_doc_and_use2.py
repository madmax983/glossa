import re

with open('src/tools/mod.rs', 'r') as f:
    mod = f.read()

# I forgot that tools::find_glossa_binary is only used in tools::tester and tools::runner inside the tests!
# So it needs to be `#[cfg(test)]` or `#[allow(dead_code)]` because it's only used by tests in submodules.
# Oh actually `find_glossa_binary` is used inside `#[cfg(test)] mod tests` in runner and tester.

new_mod = mod.replace("pub(crate) fn find_glossa_binary() -> String {", "#[cfg(test)]\n#[allow(dead_code)]\npub(crate) fn find_glossa_binary() -> String {")

with open('src/tools/mod.rs', 'w') as f:
    f.write(new_mod)
