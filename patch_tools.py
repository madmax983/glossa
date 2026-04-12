import re

with open('src/tools/mod.rs', 'r') as f:
    content = f.read()

# Make the tools module strictly encapuslated, BUT we have to expose the functions
# used by main.rs and tests/ to be available directly on `glossa::tools::...` or `glossa::...`
# If we change `pub mod <name>;` to `pub(crate) mod <name>;`, we must add
# `pub use <name>::<function>;` to `src/tools/mod.rs` for the things needed by `main.rs` and `tests/`.

# 1. replace all `pub mod ` with `pub(crate) mod `
new_content = re.sub(r'pub mod ([a-zA-Z0-9_]+);', r'pub(crate) mod \1;', content)

# 2. Add `pub use` for what's needed by `main.rs` and `tests/`.
pub_uses = """
// Re-export what is needed by main.rs and integration tests
pub use cli::{Cli, Commands};
pub use dictionary::lookup_word;
pub use repl::run_repl;
pub use runner::{build_file, check_file, highlight_file, bard_file, run_file, report_file};
pub use tester::run_tests;
pub use highlight::highlight;
pub use narrator::tell_tale;

#[cfg(feature = "nova")]
pub use mentor::run_mentor;
#[cfg(feature = "nova")]
pub use mosaic::{run_mosaic, run_mosaic_inner};
#[cfg(feature = "nova")]
pub use cartographer::run_map;
#[cfg(feature = "nova")]
pub use labyrinth::run_labyrinth;
#[cfg(feature = "nova")]
pub use weave::run_weave;
#[cfg(feature = "nova")]
pub use alchemist::{run_alchemist, transpile_to_python};
#[cfg(feature = "nova")]
pub use papyrus::run_papyrus;
#[cfg(feature = "nova")]
pub use auditor::run_auditor;
#[cfg(feature = "nova")]
pub use interpreter::Interpreter;
"""

new_content += pub_uses

with open('src/tools/mod.rs', 'w') as f:
    f.write(new_content)
