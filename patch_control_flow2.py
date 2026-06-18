import re

with open('src/semantic/control_flow.rs', 'r') as f:
    content = f.read()

content = content.replace(
    'Err(crate::errors::AssemblyError::MissingVerb) => {',
    'Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb)) => {'
)
content = content.replace(
    'Err(e) => return Err(GlossaError::AssemblyError(e)),',
    'Err(e) => return Err(e),'
)

with open('src/semantic/control_flow.rs', 'w') as f:
    f.write(content)
