#!/bin/bash
cargo test
cargo install cargo-tarpaulin
cargo tarpaulin --ignore-tests
