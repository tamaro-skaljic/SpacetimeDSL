@echo off
cargo fmt --all -- --check

cd derive-input
cargo clippy --all-targets --all-features
        
cd ../derive
cargo clippy --all-targets --all-features
        
cd ..
cargo clippy --all-targets --all-features

cd example
cargo clippy --all-targets --all-features
