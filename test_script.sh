#!/bin/bash
cat src/semantic/types.rs | grep "pub enum GlossaType" -A 40
