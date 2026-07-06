# Using python script to submit due to the long multiline message format of the submit tool
import json
with open("submit_args.json", "w") as f:
    json.dump({
        "branch_name": "codex-haruspex-adr",
        "title": "📜 Codex: ADR 022 & Architecture Diagram Update",
        "commit_message": "docs: add ADR 022 and update architecture diagram for haruspex",
        "description": "🧠 **Decision:** Recorded the addition of the Haruspex AST visualizer tool.\n🗺️ **Visuals:** Updated C4 Component diagram to show Haruspex within the Developer Experience boundary and its connection to the Semantic Analyzer.\n🔗 **Link:** See `docs/adr/022-haruspex-ast-visualizer.md`."
    }, f)
