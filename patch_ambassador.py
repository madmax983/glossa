content = open('src/tools/ambassador.rs').read()
content = content.replace("return Err(e);", "return Err(e.into());")
open('src/tools/ambassador.rs', 'w').write(content)
