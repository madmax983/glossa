with open("src/tools/alchemist.rs", "r") as f:
    content = f.read()

content = content.replace(
    """        let _ = write!(&mut out, "{}    pass\\n", ind);""",
    """        let _ = writeln!(&mut out, "{}    pass", ind);"""
)

content = content.replace(
    """        let _ = write!(&mut out, "{}else:\\n", ind);""",
    """        let _ = writeln!(&mut out, "{}else:", ind);"""
)

content = content.replace(
    """            let _ = write!(&mut out, "{}    pass\\n", ind);""",
    """            let _ = writeln!(&mut out, "{}    pass", ind);"""
)

content = content.replace(
    """            let _ = write!(&mut out, "{}        pass\\n", ind);""",
    """            let _ = writeln!(&mut out, "{}        pass", ind);"""
)

with open("src/tools/alchemist.rs", "w") as f:
    f.write(content)
print("Replaced write! newline errors")
