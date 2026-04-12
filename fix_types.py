import re

with open('src/semantic/types.rs', 'r') as f:
    content = f.read()

clone_drop_impl = """
impl Clone for GlossaType {
    fn clone(&self) -> Self {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            GlossaType::Unit => GlossaType::Unit,
            GlossaType::Number => GlossaType::Number,
            GlossaType::String => GlossaType::String,
            GlossaType::Boolean => GlossaType::Boolean,
            GlossaType::Function { params, returns } => GlossaType::Function { params: params.clone(), returns: returns.clone() },
            GlossaType::Struct { name, gender, fields } => GlossaType::Struct { name: name.clone(), gender: *gender, fields: fields.clone() },
            GlossaType::List(t) => GlossaType::List(t.clone()),
            GlossaType::Map(key, value) => GlossaType::Map(key.clone(), value.clone()),
            GlossaType::Set(t) => GlossaType::Set(t.clone()),
            GlossaType::Option(t) => GlossaType::Option(t.clone()),
            GlossaType::Result(ok, err) => GlossaType::Result(ok.clone(), err.clone()),
            GlossaType::Unknown => GlossaType::Unknown,
        })
    }
}

impl Drop for GlossaType {
    fn drop(&mut self) {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            match self {
                GlossaType::Function { params, returns } => {
                    let _ = std::mem::take(params);
                    let _ = std::mem::replace(returns, Box::new(GlossaType::Unit));
                }
                GlossaType::Struct { fields, .. } => {
                    let _ = std::mem::take(fields);
                }
                GlossaType::List(t) | GlossaType::Set(t) | GlossaType::Option(t) => {
                    let _ = std::mem::replace(&mut **t, GlossaType::Unit);
                }
                GlossaType::Map(key, value) => {
                    let _ = std::mem::replace(&mut **key, GlossaType::Unit);
                    let _ = std::mem::replace(&mut **value, GlossaType::Unit);
                }
                GlossaType::Result(ok, err) => {
                    let _ = std::mem::replace(&mut **ok, GlossaType::Unit);
                    let _ = std::mem::replace(&mut **err, GlossaType::Unit);
                }
                _ => {}
            }
        })
    }
}
"""

content = re.sub(r'impl Clone for GlossaType \{.*?\n\}\n', '', content, flags=re.DOTALL)
content = re.sub(r'impl Drop for GlossaType \{.*?\n\}\n', '', content, flags=re.DOTALL)

content = content.replace("    Unknown,\n}\n", "    Unknown,\n}\n" + clone_drop_impl)

with open('src/semantic/types.rs', 'w') as f:
    f.write(content)
