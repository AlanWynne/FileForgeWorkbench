# Reads the most recent FileForgeWorkbench run log and prints the [menus-tab]
# diagnostic lines. Usage: pwsh -NoProfile -File tools\powershell\read-menus-tab-log.ps1
$log = Get-ChildItem "$env:LOCALAPPDATA\FileForgeWorkbench\logs\*.log" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime |
    Select-Object -Last 1
if ($null -eq $log) {
    Write-Output "NO LOG FOUND"
    return
}
Write-Output "LOG: $($log.FullName)"
$lines = Get-Content $log.FullName | Select-String -Pattern 'menus-tab'
if (-not $lines) {
    Write-Output "NO [menus-tab] LINES (diagnostic did not fire this run)"
    return
}
$lines | Select-Object -Last 25 | ForEach-Object { $_.Line }
