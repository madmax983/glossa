import re

with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace("glossa::tools::cli::{Cli, Commands}", "glossa::tools::{Cli, Commands}")
content = content.replace("glossa::tools::dictionary::lookup_word", "glossa::tools::lookup_word")
content = content.replace("glossa::tools::repl::run_repl", "glossa::tools::run_repl")
content = content.replace("glossa::tools::runner::{", "glossa::tools::{")
content = content.replace("glossa::tools::mentor::run_mentor", "glossa::tools::run_mentor")
content = content.replace("glossa::tools::tester::run_tests", "glossa::tools::run_tests")
content = content.replace("glossa::tools::mosaic::run_mosaic", "glossa::tools::run_mosaic")
content = content.replace("glossa::tools::cartographer::run_map", "glossa::tools::run_map")
content = content.replace("glossa::tools::labyrinth::run_labyrinth", "glossa::tools::run_labyrinth")
content = content.replace("glossa::tools::weave::run_weave", "glossa::tools::run_weave")
content = content.replace("glossa::tools::alchemist::run_alchemist", "glossa::tools::run_alchemist")
content = content.replace("glossa::tools::papyrus::run_papyrus", "glossa::tools::run_papyrus")
content = content.replace("glossa::tools::auditor::run_auditor", "glossa::tools::run_auditor")

with open('src/main.rs', 'w') as f:
    f.write(content)
