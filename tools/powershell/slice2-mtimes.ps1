<#
.SYNOPSIS Read-only: newest write times across the slice2 worktree source + i18n
  data + any commits, to judge crate1 liveness. Writes a log to read back.
.NOTES One-off diagnostic. Read-only.
#>
$ErrorActionPreference = 'Continue'
$wt = 'C:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice2'
$log = 'C:\workspace\apps\FileForgeWorkbench\tools\logs\slice2-mtimes.txt'
$now = Get-Date

$out = New-Object System.Collections.Generic.List[string]
$out.Add("=== slice2 liveness @ " + $now.ToString('HH:mm:ss') + " ===")

# commits on the branch ahead of main
$commits = & git -C $wt log --oneline main..HEAD 2>&1
$out.Add("")
$out.Add("-- commits ahead of main --")
if (-not $commits) { $out.Add("  (none yet)") } else { @($commits) | ForEach-Object { $out.Add("  " + $_) } }

# newest 12 source/data writes (exclude target + .git)
$out.Add("")
$out.Add("-- newest writes (crates + i18n, excl target) --")
$dirs = @(
    (Join-Path $wt 'crates\ff-catalog-dialog'),
    (Join-Path $wt 'crates\ff-i18n\i18n\en'),
    (Join-Path $wt 'crates\ff-i18n\src')
)
$files = foreach ($d in $dirs) {
    if (Test-Path $d) {
        Get-ChildItem $d -Recurse -File -Include *.rs, *.ftl -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -notmatch '\\target\\' }
    }
}
$files | Sort-Object LastWriteTime -Descending | Select-Object -First 12 | ForEach-Object {
    $age = [int]($now - $_.LastWriteTime).TotalSeconds
    $rel = $_.FullName.Substring($wt.Length).TrimStart('\')
    $out.Add(("  {0,6}s  {1}" -f $age, $rel))
}

# uncommitted changes
$out.Add("")
$out.Add("-- uncommitted (git status) --")
$st = & git -C $wt status --short 2>&1
if (-not $st) { $out.Add("  (clean)") } else { @($st) | ForEach-Object { $out.Add("  " + $_) } }

$out.Add("")
$out.Add("=== end ===")
Set-Content -Path $log -Value $out -Encoding utf8
