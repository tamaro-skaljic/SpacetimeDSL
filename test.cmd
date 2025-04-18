@echo off

echo Building module...
echo.
cd test
spacetime publish spacetimedsl
echo.

echo Creating Debug Output ...
echo.

echo Expanding test library ...
echo.
cargo expand --lib > ../debug/output/lib.expanded.rs

cd ../debug
echo Creating AST of initial test lib.
echo.
cargo run -- ../test/src/lib.rs > output/lib.rs.ast
echo.

echo Creating AST of expanded test lib.
echo.
cargo run -- output/lib.expanded.rs > output/lib.expanded.rs.ast

echo Testing module...
echo.
spacetime call spacetimedsl test
echo.

echo Showing logs...
echo.
spacetime logs spacetimedsl
echo.

echo Cleaning up module ...
echo.
spacetime delete spacetimedsl
echo.
