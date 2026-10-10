<#
.SYNOPSIS
  Reusable FFWB evidence + scoped-check helper. All output goes to files under
  tools/logs/ so the caller can read results even when the interactive terminal
  mangles stdout. Invoke with a SINGLE short task word so the command line stays
  clean (the long/complex logic lives here, per .kiro/steering/tooling.md).

.DESCRIPTION
  Tasks:
    status  - write a timestamped filesystem snapshot (recent mtimes under the
              dataset/shell work area + key file sizes) to tools/logs/ff-status.txt.
              Pure read-only; use to judge whether a background agent is still
              writing files (liveness evidence) without touching cargo.
    check   - run the scoped BRC.4 / RC.B.8(b-d) compile checks
              (cargo check -p ff-idcams -p ff-desktop -p ff-dscatalog -p ff-vfs
              -p ff-volume) to tools/logs/ff-check.txt. Exit status appended.
    test    - run the scoped tests (cargo test -p ff-idcams; cargo test -p
              ff-desktop) to tools/logs/ff-test.txt.
    clippy  - cargo clippy -p ff-idcams -p ff-desktop -- -D warnings to
              tools/logs/ff-clippy.txt.
    fmt     - cargo fmt (writes), result note to tools/logs/ff-fmt.txt.

.NOTES
  Reusable project tool (tools/powershell/). Read-only except `fmt` (formats) and
  the cargo build cache. Safe to re-run. Logs are overwritten each run.
#>
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('status', 'check', 'test', 'clippy', 'fmt')]
    [string]$Task
)

$ErrorActionPreference = 'Continue'
$repo = 'C:\workspace\VSC\FileForgeWorkbench'
$logs = Join-Path $repo 'tools\logs'
if (-not (Test-Path $logs)) { New-Item -ItemType Directory -Path $logs -Force | Out-Null }

function Write-Log($name, $lines) {
    $path = Join-Path $logs $name
    Set-Content -Path $path -Value $lines -Encoding utf8
}

switch ($Task) {
    'status' {
        $now = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
        $out = @("=== ff-check status @ $now ===", '')
        # Recent-mtime snapshot of the dataset/shell work area (the files the
        # record-aware-SAVE agents touch), newest first.
        $paths = @(
            'crates\ff-idcams\src',
            'crates\ff-desktop\src\shell',
            'crates\ff-vfs\src',
            'crates\ff-dscatalog\src'
        )
        foreach ($p in $paths) {
            $full = Join-Path $repo $p
            $out += "--- $p (newest 8 by LastWriteTime) ---"
            $items = Get-ChildItem -Path $full -File -Recurse -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending |
                Select-Object -First 8
            foreach ($i in $items) {
                $out += ('{0}  {1}' -f $i.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss'), $i.FullName.Substring($repo.Length + 1))
            }
            $out += ''
        }
        Write-Log 'ff-status.txt' $out
    }
    'check' {
        Push-Location $repo
        $o = & cargo check -p ff-idcams -p ff-desktop -p ff-dscatalog -p ff-vfs -p ff-volume --all-targets 2>&1
        $o += ''
        $o += ("=== cargo check exit code: {0} ===" -f $LASTEXITCODE)
        Write-Log 'ff-check.txt' $o
        Pop-Location
    }
    'test' {
        Push-Location $repo
        $o = & cargo test -p ff-idcams 2>&1
        $o += ''
        $o += ("=== cargo test -p ff-idcams exit code: {0} ===" -f $LASTEXITCODE)
        $o += ''
        $o += '================ ff-desktop ================'
        $o2 = & cargo test -p ff-desktop 2>&1
        $o += $o2
        $o += ''
        $o += ("=== cargo test -p ff-desktop exit code: {0} ===" -f $LASTEXITCODE)
        Write-Log 'ff-test.txt' $o
        Pop-Location
    }
    'clippy' {
        Push-Location $repo
        $o = & cargo clippy -p ff-idcams -p ff-desktop -- -D warnings 2>&1
        $o += ''
        $o += ("=== cargo clippy exit code: {0} ===" -f $LASTEXITCODE)
        Write-Log 'ff-clippy.txt' $o
        Pop-Location
    }
    'fmt' {
        Push-Location $repo
        $o = & cargo fmt 2>&1
        $o += ("=== cargo fmt exit code: {0} ===" -f $LASTEXITCODE)
        Write-Log 'ff-fmt.txt' $o
        Pop-Location
    }
}
