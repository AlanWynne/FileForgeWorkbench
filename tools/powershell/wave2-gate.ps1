# Scoped gate for Decomposition Wave 2 (ff-toolchain-panel extraction).
# Writes progress to tools\logs\wave2-gate.txt; appends EXIT markers per step.
$ErrorActionPreference = 'Continue'
$log = 'tools\logs\wave2-gate.txt'
Remove-Item $log -ErrorAction SilentlyContinue

cargo fmt --check 2>&1 | Out-File -Append $log
"FMT_EXIT=$LASTEXITCODE" | Out-File -Append $log

cargo clippy -p ff-toolchain-panel -p ff-desktop --all-targets -- -D warnings 2>&1 | Out-File -Append $log
"CLIPPY_EXIT=$LASTEXITCODE" | Out-File -Append $log

cargo nextest run -p ff-toolchain-panel -p ff-desktop 2>&1 | Out-File -Append $log
"NEXTEST_EXIT=$LASTEXITCODE" | Out-File -Append $log

"DONE=1" | Out-File -Append $log
