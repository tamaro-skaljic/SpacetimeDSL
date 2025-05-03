@echo off

echo Publishing lib_derive-output...
cd derive-output
cargo publish --allow-dirty
cd ..
