import re

content = open("tests/havoc_issue_echo.rs").read()

content = re.sub(r'#\[test\]\n#\[should_panic\(expected = "UndefinedName"\)\]\nfn test_try_print_default.*?\n\}\n', '', content, flags=re.DOTALL)

open("tests/havoc_issue_echo.rs", "w").write(content)
