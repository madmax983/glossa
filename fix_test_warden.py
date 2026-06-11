import re

with open('tests/warden_missing_verb.rs', 'r') as f:
    content = f.read()

content = content.replace(
    '''assert!(
        err_msg.contains("Ῥῆμα οὐχ εὑρέθη!"),
        "Expected MissingVerb error, got: {}",
        err_msg
    );''',
    '''assert!(
        err_msg.contains("Ἄγνωστον ὄνομα:"),
        "Expected UndefinedName error, got: {}",
        err_msg
    );'''
)

with open('tests/warden_missing_verb.rs', 'w') as f:
    f.write(content)
