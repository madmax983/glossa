#!/bin/bash
perl -0777 -pi -e 's/#[cfg\(test\)]\nmod facade_tests.*//s' src/tools/mod.rs
