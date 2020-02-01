#!/bin/bash
if [[ "$OSTYPE" == "linux-gnu" ]]; then
        feature="vulkan"
elif [[ "$OSTYPE" == "darwin"* ]]; then
        feature="metal"
elif [[ "$OSTYPE" == "cygwin" ]]; then
        feature="vulkan"
elif [[ "$OSTYPE" == "msys" ]]; then
        feature="vulkan"
elif [[ "$OSTYPE" == "win32" ]]; then
        feature="vulkan"
fi

BUILD_ENABLED=1 RUST_BACKTRACE=1 RUST_LOG=clockwork=info cargo run --features=$feature