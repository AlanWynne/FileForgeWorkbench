# One-off: report total line count and first #[cfg(test)] line for key CR-NR-109 files.
# Output: writes a table to tools/logs/crnr109-linecount.log (read it back).
$root = 'c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr109-locale-alias\crates\ff-desktop\src'
$files = @(
    "$root\shell\environment.rs",
    "$root\shell\environment_contract.rs",
    "$root\shell\alias_reader.rs",
    "$root\shell\alias_overlay.rs",
    "$root\shell\locale_resolver.rs",
    "$root\shell\construct.rs",
    "$root\shell\dispatch.rs",
    "$root\shell\commands_menu.rs",
    "$root\shell\tests_locale_menu.rs",
    "$root\menu_workspace\render.rs",
    "$root\menu_workspace\render_layout.rs",
    "$root\menu_workspace\i18n_keys.rs",
    "$root\command_config\mod.rs"
)
$out = foreach ($f in $files) {
    if (Test-Path $f) {
        $total = (Get-Content $f).Count
        $cfg = Select-String -Path $f -Pattern '#\[cfg\(test\)\]' | Select-Object -First 1
        $line = if ($cfg) { $cfg.LineNumber } else { 'none' }
        $nontest = if ($cfg) { $cfg.LineNumber - 1 } else { $total }
        [PSCustomObject]@{ File = (Split-Path $f -Leaf); Total = $total; CfgTestAt = $line; NonTest = $nontest }
    } else {
        [PSCustomObject]@{ File = (Split-Path $f -Leaf); Total = 'MISSING'; CfgTestAt = '-'; NonTest = '-' }
    }
}
$out | Format-Table -AutoSize | Out-String -Width 200
