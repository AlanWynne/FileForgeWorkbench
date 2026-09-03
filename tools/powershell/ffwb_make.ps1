cargo fmt -- --check
cargo fmt 
cargo clippy --workspace -- -D warnings
cargo build  --workspace --release 2>&1 | Select-String 'error|warning' 
cargo test  2>&1 | Select-String '(?i)error|warning' 
.\target\debug\ffwb.exe