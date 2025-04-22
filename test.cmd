@echo off

echo Building module...
echo.
cd example
spacetime publish spacetimedsl
echo.

echo Testing module...
echo.
spacetime call spacetimedsl tester
echo.

echo Showing logs...
echo.
spacetime logs spacetimedsl
echo.

echo Cleaning up module ...
echo.
spacetime delete spacetimedsl
echo.

echo Creating Debug Output ...
echo.

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
