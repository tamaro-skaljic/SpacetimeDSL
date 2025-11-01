cd example
set RUSTFLAGS="-Zmacro-backtrace"; cargo +nightly expand > ..\debug-helper\output\lib.expanded.rs
cd ..\debug-helper
cargo run -- ..\example\src\lib.rs > output\lib.rs.ast
cd ..
