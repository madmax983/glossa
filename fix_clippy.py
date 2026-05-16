with open('src/semantic/conversion/tests.rs', 'r') as f:
    content = f.read()

content = content.replace('mod tests {', '#[allow(clippy::module_inception)]\nmod tests {')

with open('src/semantic/conversion/tests.rs', 'w') as f:
    f.write(content)
