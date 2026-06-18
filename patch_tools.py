import re

with open('src/tools/mod.rs', 'r') as f:
    content = f.read()

# Replace read_line_bounded
new_fn = """pub(crate) fn read_line_bounded<R: BufRead>(
    reader: &mut R,
    buf: &mut String,
    limit: usize,
) -> Result<usize, std::io::Error> {
    use std::io::Read;
    reader.by_ref().take(limit as u64).read_line(buf)
}"""

# Need to replace the old block correctly
old_fn = """pub(crate) fn read_line_bounded<R: BufRead>(
    reader: &mut R,
    buf: &mut String,
    limit: usize,
) -> Result<usize, std::io::Error> {
    use std::io::Read;
    let mut bytes_read = 0;
    let mut byte_buf = Vec::new();
    for byte_res in reader.bytes() {
        let byte = byte_res?;
        byte_buf.push(byte);
        bytes_read += 1;
        if byte == b'\\n' || bytes_read >= limit {
            break;
        }
    }
    buf.push_str(&String::from_utf8_lossy(&byte_buf));
    Ok(bytes_read)
}"""

content = content.replace(old_fn, new_fn)

with open('src/tools/mod.rs', 'w') as f:
    f.write(content)

with open('src/tools/tester.rs', 'r') as f:
    tester_content = f.read()

# std::env::current_exe() issues in tests:
# A fallback that defaults to current_exe() causes recursion panics when running tests because current_exe is the test runner.
# We should instead error out gracefully or panic with a clear message if glossa isn't found.
# Actually, the previous implementation fell back to `cargo run` maybe?
# Let's see what the original looked like.
