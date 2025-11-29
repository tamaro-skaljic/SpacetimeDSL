@echo off
cargo fmt --all -- --check

cd derive-input
REM cargo clippy --fix --allow-dirty --all-features
        
cd ../derive
cargo clippy --fix --allow-dirty --all-features
        
cd ..
REM cargo clippy --fix --allow-dirty --all-features

cd examples/test
cargo clippy --fix --allow-dirty --all-features

cd ../blackholio
cargo clippy --fix --allow-dirty --all-features
