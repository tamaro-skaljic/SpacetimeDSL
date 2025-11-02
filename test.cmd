@echo off

echo Building module...
echo.
cd examples\test
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

