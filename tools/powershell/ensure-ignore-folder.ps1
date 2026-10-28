<#
.SYNOPSIS
  Ensure the project's _ignore scaffold exists and is git-ignored.

.DESCRIPTION
  Idempotent, NON-DESTRUCTIVE. Moves nothing. Run on every session start (via a
  SessionStart hook) and safe to run by hand any number of times:

    1. Creates the <repo>/_ignore/ folder if it is missing (with a .gitkeep so
       the empty folder is retained locally).
    2. Ensures .gitignore contains a "_ignore/" entry (ignores the folder and all
       subfolders). Appends the entry under a labelled section if absent.

  It does NOT move, delete, or reclassify any file. Relocating stray files into
  _ignore/ is the separate, deliberate once-off tool sweep-to-ignore.ps1.

.PARAMETER RepoRoot
  Absolute path to the project root. Defaults to the parent of tools/ (two levels
  up from this script), so the hook can call it with no argument.

.OUTPUTS
  Writes a log to <repo>/_ignore/logs/ensure-ignore-folder.txt and echoes it.
  Exit 0 always on success; non-zero only on an unexpected IO error.
#>
[CmdletBinding()]
param(
    [string]$RepoRoot
)

$ErrorActionPreference = 'Stop'

if (-not $RepoRoot -or $RepoRoot.Trim() -eq '') {
    # tools/powershell/ -> tools/ -> <repo>
    $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}

$ignoreDir = Join-Path $RepoRoot '_ignore'
$logDir    = Join-Path $ignoreDir 'logs'
$gitignore = Join-Path $RepoRoot '.gitignore'

$lines = New-Object System.Collections.Generic.List[string]
function Note([string]$m) { $lines.Add($m) }

Note("ensure-ignore-folder: repo = $RepoRoot")

# 1. Ensure _ignore/ exists (and its logs/ subfolder + a .gitkeep).
if (-not (Test-Path -LiteralPath $ignoreDir)) {
    New-Item -ItemType Directory -Path $ignoreDir -Force | Out-Null
    Note("CREATED _ignore/")
} else {
    Note("_ignore/ already present")
}
if (-not (Test-Path -LiteralPath $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    Note("CREATED _ignore/logs/")
}
$gitkeep = Join-Path $ignoreDir '.gitkeep'
if (-not (Test-Path -LiteralPath $gitkeep)) {
    New-Item -ItemType File -Path $gitkeep -Force | Out-Null
    Note("CREATED _ignore/.gitkeep")
}

# 2. Ensure .gitignore ignores _ignore/ (the folder and every subfolder).
$entry  = '_ignore/'
$header = '# -- Session-ignored artefacts (logs, scratch, non-committed data) --'
if (Test-Path -LiteralPath $gitignore) {
    $content = Get-Content -LiteralPath $gitignore
    $has = $false
    foreach ($l in $content) {
        if ($l.Trim() -eq $entry -or $l.Trim() -eq '/_ignore/' -or $l.Trim() -eq '_ignore') {
            $has = $true; break
        }
    }
    if ($has) {
        Note("_ignore/ already in .gitignore")
    } else {
        Add-Content -LiteralPath $gitignore -Value ("`r`n" + $header + "`r`n" + $entry)
        Note("APPENDED '$entry' to .gitignore")
    }
} else {
    Set-Content -LiteralPath $gitignore -Value ($header + "`r`n" + $entry)
    Note("CREATED .gitignore with '$entry'")
}

Note("ensure-ignore-folder: done")

# Write the log INTO _ignore/logs (which is itself ignored) and echo it.
$logPath = Join-Path $logDir 'ensure-ignore-folder.txt'
$lines | Set-Content -LiteralPath $logPath -Encoding utf8
$lines | ForEach-Object { Write-Output $_ }
exit 0
