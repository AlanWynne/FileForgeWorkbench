cargo fmt >C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.fmt.log  2>>&1
cargo build  --workspace >C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.build.log  2>>&1
cargo test   --workspace >C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.test.log  2>>&1
cargo clippy --workspace >C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.clippy.log 2>>&1
cargo nextest run --workspace > C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.nextest.log 2>>&1
c:\tools\powershell7\pwsh -ExecutionPolicy Bypass -File C:\workspace\VSC\FileForgeWorkbench\tools\powershell\verify.ps1 >C:\workspace\VSC\FileForgeWorkbench\tools\logs\verify.log 2>>&1
type C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.fmt.log 
type C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.build.log
type C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.test.log
type C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.clippy.log
type C:\workspace\VSC\FileForgeWorkbench\tools\logs\cargo.nextest.log
type C:\workspace\VSC\FileForgeWorkbench\tools\logs\verify.log