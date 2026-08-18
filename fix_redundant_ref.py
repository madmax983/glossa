import re

with open("src/semantic/conversion.rs", "r") as f:
    content = f.read()

# Replace '&var_name' with 'var_name' where it matches the line 512
content = content.replace('            "Τὸ «{}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ",\n            &var_name', '            "Τὸ «{}» ἀμετάβλητόν ἐστιν — χρῆσον μετά πρὸ τοῦ ὁρισμοῦ",\n            var_name')

with open("src/semantic/conversion.rs", "w") as f:
    f.write(content)
