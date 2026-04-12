import re

with open('src/semantic/model.rs', 'r') as f:
    content = f.read()

content = content.replace("#[derive(Clone)]\npub struct AnalyzedMethod", "pub struct AnalyzedMethod")

clone_drop_impl = """
impl Clone for AnalyzedMethod {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || AnalyzedMethod {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            return_type: self.return_type.clone(),
        })
    }
}
"""
content = content.replace("    pub return_type: Option<GlossaType>,\n}\n", "    pub return_type: Option<GlossaType>,\n}\n" + clone_drop_impl)

with open('src/semantic/model.rs', 'w') as f:
    f.write(content)
