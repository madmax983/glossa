import sys

content = open('src/semantic/conversion.rs').read()

content = content.replace('scope.lookup_binding(var_name)', 'scope.lookup_binding(var_name.as_str())')
content = content.replace('scope.mark_used(var_name)', 'scope.mark_used(var_name.as_str())')

open('src/semantic/conversion.rs', 'w').write(content)
