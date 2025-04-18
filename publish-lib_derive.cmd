@echo off

echo Publishing lib_derive...
cd derive
cargo publish --allow-dirty
cd ..
