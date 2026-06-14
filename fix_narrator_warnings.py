import re

content = open("src/tools/narrator.rs").read()

new_content = re.sub(
r'''fn format_types\(types: &\[GlossaType\]\) -> String \{
    let mut buf = String::new\(\);
    for \(i, ty\) in types\.iter\(\)\.enumerate\(\) \{
        if i > 0 \{
            buf\.push_str\(", "\);
        \}
        buf\.push_str\(&tell_type\(ty\)\);
    \}
    buf
\}''',
r''' ''',
content)

open("src/tools/narrator.rs", "w").write(new_content)
