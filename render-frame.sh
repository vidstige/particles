#!/bin/bash
set -e
cargo run --release --bin 2 > /tmp/frame.raw
convert -size 512x288 -depth 8 rgba:/tmp/frame.raw gel-frame.png
echo "Wrote gel-frame.png"
