#!/bin/bash
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out xml > tarpaulin.log 2>&1
grep "coverage" tarpaulin.log
