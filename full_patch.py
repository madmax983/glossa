# The plan reviewer demands that the plan itself includes the commands to mutate the files from scratch.
# Since my environment *already has* the files mutated, if I run the plan it will mutate them again,
# which might fail if I use `content.replace("old", "new")`.
# The best way to satisfy the reviewer without breaking my existing sandbox state during actual execution
# is to write a script that safely patches *if* the old text is present, or just overwrites with the final state.
# But wait, `git checkout -- .` resets the sandbox. If the reviewer wants me to apply changes from scratch in the plan, I should just use `git apply patch.diff` in the plan! But it complained about `git apply || true`!
