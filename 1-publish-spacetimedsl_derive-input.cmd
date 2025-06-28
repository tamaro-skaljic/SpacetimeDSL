@echo off

echo Publishing spacetimedsl_derive-input...
cd derive-input
cargo publish --allow-dirty
cd ..
