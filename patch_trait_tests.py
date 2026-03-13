import re

with open("tests/trait_tests.rs", "r") as f:
    content = f.read()

content = content.replace('''που show.''', '''που show λέγε.''')
content = content.replace('''p1ου show.''', '''p1ου show λέγε.''')
content = content.replace('''p2ου display.''', '''p2ου display λέγε.''')
content = content.replace('''p2ου show.''', '''p2ου show λέγε.''')
content = content.replace('''pου show.''', '''pου show λέγε.''')
content = content.replace('''dου speak.''', '''dου speak λέγε.''')
content = content.replace('''cου speak.''', '''cου speak λέγε.''')

with open("tests/trait_tests.rs", "w") as f:
    f.write(content)
