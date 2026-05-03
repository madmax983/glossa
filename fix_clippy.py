content = open('src/semantic/conversion.rs', 'r').read()
import re
new_content = re.sub(r'let _ = !scope\.is_defined.*?"selfους";', '', content, flags=re.DOTALL)
new_content = re.sub(r'!scope\.is_defined.*?selfους";', '', new_content, flags=re.DOTALL)

with open('src/semantic/conversion.rs', 'w') as f:
    f.write(new_content)
