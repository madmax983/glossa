#!/bin/bash
cat << 'EOF2' > test_script.γλ
εἶδος String ὁρίζειν { }. τέλος.
EOF2
cargo run --bin glossa -- test_script.γλ
