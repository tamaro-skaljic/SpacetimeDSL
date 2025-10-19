@echo off
cargo fmt --all -- --check

cd derive-input
cargo clippy --fix --allow-dirty --all-features
        
cd ../derive
cargo clippy --fix --allow-dirty --all-features
        
cd ..
cargo clippy --fix --allow-dirty --all-features

cd example
cargo clippy --fix --allow-dirty --all-features
