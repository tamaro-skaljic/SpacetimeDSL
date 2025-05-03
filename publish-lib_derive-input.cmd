@echo off

echo Publishing lib_derive-input...
cd derive-input
cargo publish --allow-dirty
cd ..
