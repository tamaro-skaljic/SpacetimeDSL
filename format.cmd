@echo off
cargo fmt --all -- --check

cd derive-input
cargo clippy --all-targets --all-features || echo "Clippy warnings in derive-input"
        
cd ../derive
cargo clippy --all-targets --all-features || echo "Clippy warnings in derive"
        
cd ..
cargo clippy --all-targets --all-features || echo "Clippy warnings in main lib"
