content = open('src/tools/ambassador.rs').read()
content = content.replace('let _ = writeln!(output, "");', 'let _ = writeln!(output);')
open('src/tools/ambassador.rs', 'w').write(content)
