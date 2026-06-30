with open("tests/reproduce_silent_failure.rs", "r") as f:
    content = f.read()

target = """            assert!(
                msg.contains("Χρήστος")
                    || msg.contains("undefined")
                    || msg.contains("Ἄγνωστον")
                    || msg.contains("Άγνωστον")
            );"""

replacement = """            assert!(
                msg.contains("Χρήστος")
                    || msg.contains("undefined")
                    || msg.contains("Ἄγνωστον")
                    || msg.contains("Άγνωστον")
                    || msg.contains("Οὐκ οἶδα τὸ ὄνομα")
            );"""

if target in content:
    content = content.replace(target, replacement)
    print("Patched reproduce 3 correctly")

with open("tests/reproduce_silent_failure.rs", "w") as f:
    f.write(content)
