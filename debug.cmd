@echo off

cd SpacetimeDSL\example

echo Expanding example library ...
echo.
cargo expand --lib > ..\debug-helper\output\lib.expanded.rs

cd ..\debug-helper
echo Creating AST of initial example lib.
echo.
cargo run -- ..\example\src\lib.rs > output\lib.rs.ast
echo.

echo Creating AST of expanded example lib.
echo.
cargo run -- output/lib.expanded.rs > output/lib.expanded.rs.ast
