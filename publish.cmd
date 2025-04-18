@echo off

echo Publishing lib_derive...
cd lib_derive
cargo publish
echo.

echo Publishing lib...
cd ../lib
cargo publish
cd ..
