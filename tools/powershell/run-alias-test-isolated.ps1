# Diagnostic (read-only): run the failing alias test with the user-alias
# override base redirected to an EMPTY dir via FFWB_USER_ALIASES_DIR, proving
# the construction-time global overlay from %APPDATA% is the stray-row source.
# Usage: pwsh -NoProfile -NonInteractive -File run-alias-test-isolated.ps1 <emptyDir> <logFile>
param(
    [Parameter(Mandatory = $true)] [string] $EmptyDir,
    [Parameter(Mandatory = $true)] [string] $LogFile
)
$env:FFWB_USER_ALIASES_DIR = $EmptyDir
cargo test -p ff-desktop line_commands_insert_and_delete_the_working_copy -- --test-threads=1 *> $LogFile
