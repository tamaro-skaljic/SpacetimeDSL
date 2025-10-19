@echo off
cargo fmt --all -- --check

cd derive-input
cargo clippy --fix --all-features
        
cd ../derive
cargo clippy --fix --all-features
        
cd ..
cargo clippy --fix --all-features

cd example
cargo clippy --fix --all-features
