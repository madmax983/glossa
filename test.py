import sys
with open('src/semantic/conversion.rs', 'r') as f:
    if 'extract_subject_fallback' in f.read():
        print("FOUND")
    else:
        print("NOT FOUND")
