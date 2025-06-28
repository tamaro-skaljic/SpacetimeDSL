@echo off

echo Publishing spacetimedsl_derive...
cd derive
cargo publish --allow-dirty
cd ..
