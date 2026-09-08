cd C:\workspace\VSC\FileForgeWorkbench
Remove-Item -Path "C:\workspace\VSC\FileForgeWorkbench\tools\logs\*.log" -ErrorAction SilentlyContinue
cargo fmt 1>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.fmt.stdout.log 2>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.fmt.stderr.log
cargo check 1>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.check.stdout.log 2>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.check.stderr.log
cargo clippy --workspace 1>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.clippy.stdout.log 2>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.clippy.stderr.log
cargo build  --workspace 1>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.build.stdout.log 2>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.build.stderr.log
cargo test  1>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.test.stdout.log 2>C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.test.stderr.log
Get-ChildItem -Path "C:\workspace\VSC\FileForgeWorkbench\tools\logs" -Filter "cargo.*.log" | Select-String -Pattern "^error","^warning" | Out-File C:\workspace\VSC\FileForgeWorkbench\tools\logs\ai-review.log