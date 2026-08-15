#!/bin/bash
# Find functions that are longer than 50 lines
for file in $(find src -name "*.rs"); do
  awk '
    /^ *(pub )?fn / {
      if (in_fn) {
        if (line_count > 50) {
          print file ":" start_line " - " fn_name " (" line_count " lines)"
        }
      }
      in_fn = 1
      start_line = NR
      fn_name = $0
      sub(/ *\{.*/, "", fn_name)
      line_count = 0
    }
    in_fn {
      line_count++
    }
    END {
      if (in_fn && line_count > 50) {
        print file ":" start_line " - " fn_name " (" line_count " lines)"
      }
    }
  ' file="$file" "$file"
done
