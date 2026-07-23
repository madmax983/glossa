**[Extracted CLI command router]
**Learning:** `src/main.rs` had a God Function (`main`) of ~200 lines matching on every CLI command with feature gates.
**Action:** Extracted it into `execute_command` and `execute_nova_command` to flatten the structure and keep main tiny.

**[Extracted CLI command router]**
**Learning:** `src/main.rs` had a God Function (`main`) of ~200 lines matching on every CLI command with feature gates.
**Action:** Extracted it into `execute_command` and `execute_nova_command` to flatten the structure and keep main tiny.
