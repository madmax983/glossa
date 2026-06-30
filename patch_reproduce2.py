with open("tests/reproduce_silent_failure.rs", "r") as f:
    content = f.read()

target = """            assert!(
                msg.contains("Χρήστος")
                    || msg.contains("undefined")
                    || msg.contains("Ἄγνωστον")
                    || msg.contains("Άγνωστον")
                    || msg.contains("Οὐκ οἶδα τὸ ὄνομα")
            );"""

replacement = """            assert!(
                msg.contains("Χρήστος")
                    || msg.contains("undefined")
                    || msg.contains("Ἄγνωστον")
                    || msg.contains("Άγνωστον")
                    || msg.contains("Οὐκ οἶδα τὸ ὄνομα")
            );"""

with open("tests/reproduce_silent_failure.rs", "w") as f:
    f.write(content)
