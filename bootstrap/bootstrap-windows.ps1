<#
.SYNOPSIS
    Bootstrap the Rust toolchain for FileForge Workbench (Windows, no admin required).

.DESCRIPTION
    Downloads and installs the Rust stable toolchain into a user-level location
    under C:\tools\rust (or a custom root via -Root).  Safe to run more than once:
    skips all steps that are already complete.

    After this script succeeds, open a NEW terminal and run:
        cargo build
    from the repository root, or use tools\powershell\ffwb_make.ps1 for the
    full build-test-run workflow.

.PARAMETER Root
    Base directory for all tool installations.  Default: C:\tools

.PARAMETER Toolchain
    Rust toolchain channel to install.  Default: stable

.PARAMETER ForceReinstall
    Re-download and reinstall even if Rust is already present.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File bootstrap\bootstrap-windows.ps1
    powershell -ExecutionPolicy Bypass -File bootstrap\bootstrap-windows.ps1 -Root D:\tools
#>

[CmdletBinding()]
param(
    [string]$Root            = 'C:\tools',
    [string]$Toolchain       = 'stable',
    [switch]$ForceReinstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# === Paths ===================================================================

$RustDir    = Join-Path $Root    'rust'
$CargoHome  = Join-Path $RustDir 'cargo'
$RustupHome = Join-Path $RustDir 'rustup'
$CargoBin   = Join-Path $CargoHome 'bin'
$RustcExe   = Join-Path $CargoBin  'rustc.exe'
$CargoExe   = Join-Path $CargoBin  'cargo.exe'

$LogsDir    = Join-Path $PSScriptRoot 'logs'
$Timestamp  = Get-Date -Format 'yyyyMMdd-HHmmss'
$LogFile    = Join-Path $LogsDir "bootstrap-windows-$Timestamp.log"

# === Logging =================================================================

function New-DirIfMissing([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Write-Log {
    param(
        [Parameter(Mandatory)][string]$Message,
        [ValidateSet('INFO','WARN','ERROR')][string]$Level = 'INFO'
    )
    $line = '[{0}] [{1}] {2}' -f (Get-Date -Format 'HH:mm:ss'), $Level, $Message
    Write-Host $line
    Add-Content -Path $LogFile -Value $line
}

function Get-FileFromUrl {
    param(
        [Parameter(Mandatory)][string]$Url,
        [Parameter(Mandatory)][string]$Destination
    )
    Write-Log "Downloading: $Url"
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
    } catch {
        Write-Log "Invoke-WebRequest failed, trying WebClient: $($_.Exception.Message)" 'WARN'
        $wc = New-Object System.Net.WebClient
        try { $wc.DownloadFile($Url, $Destination) } finally { $wc.Dispose() }
    }
}

# === Initialise ==============================================================

New-DirIfMissing -Path $LogsDir
New-Item -ItemType File -Path $LogFile -Force | Out-Null

Write-Log '=== FileForge Workbench -- Rust Bootstrap (Windows) ==='
Write-Log "Root:        $Root"
Write-Log "CARGO_HOME:  $CargoHome"
Write-Log "RUSTUP_HOME: $RustupHome"
Write-Log "Toolchain:   $Toolchain"

# === Idempotency check =======================================================

if ((-not $ForceReinstall) -and (Test-Path -LiteralPath $RustcExe)) {
    Write-Log 'Rust already installed -- skipping download and install.'
    Write-Log "  rustc: $RustcExe"
} else {

    # === Download rustup-init.exe =============================================

    New-DirIfMissing -Path $Root
    New-DirIfMissing -Path (Join-Path $Root 'downloads')

    $RustupUrl  = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe'
    $RustupInit = Join-Path $Root 'downloads\rustup-init.exe'

    Get-FileFromUrl -Url $RustupUrl -Destination $RustupInit

    if (-not (Test-Path -LiteralPath $RustupInit)) {
        Write-Log 'ERROR: rustup-init.exe not found after download.' 'ERROR'
        exit 1
    }

    # === Install Rust =========================================================

    New-DirIfMissing -Path $CargoHome
    New-DirIfMissing -Path $RustupHome

    $env:CARGO_HOME  = $CargoHome
    $env:RUSTUP_HOME = $RustupHome

    Write-Log "Installing Rust $Toolchain ..."
    $proc = Start-Process -FilePath $RustupInit -ArgumentList @(
        '-y',
        '--no-modify-path',
        '--default-toolchain', $Toolchain,
        '--profile', 'default'
    ) -PassThru -Wait -NoNewWindow

    if ($proc.ExitCode -ne 0) {
        Write-Log "rustup-init.exe exited with code $($proc.ExitCode)" 'ERROR'
        exit 1
    }

    if (-not (Test-Path -LiteralPath $RustcExe)) {
        Write-Log 'ERROR: rustc.exe not found after installation.' 'ERROR'
        exit 1
    }

    Write-Log 'Rust installed successfully.'
}

# === Update user PATH ========================================================

$RegKey  = 'HKCU:\Environment'
$Current = (Get-ItemProperty -Path $RegKey -Name PATH -ErrorAction SilentlyContinue).PATH
if (-not $Current) { $Current = '' }

if ($Current -notlike "*$CargoBin*") {
    $NewPath = ($Current.TrimEnd(';') + ';' + $CargoBin).TrimStart(';')
    Set-ItemProperty -Path $RegKey -Name PATH -Value $NewPath
    Write-Log "Added $CargoBin to user PATH (HKCU)."

    # Broadcast WM_SETTINGCHANGE so open Explorer windows pick up the change
    if (-not ('NativeMethods' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeMethods {
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, int Msg, IntPtr wParam, string lParam,
        int fuFlags, int uTimeout, out IntPtr lpdwResult);
}
'@
    }
    $result = [IntPtr]::Zero
    [void][NativeMethods]::SendMessageTimeout(
        [IntPtr]0xffff, 0x001A, [IntPtr]::Zero,
        'Environment', 2, 5000, [ref]$result)
} else {
    Write-Log "User PATH already contains $CargoBin -- no change."
}

# === Verify ==================================================================

$env:CARGO_HOME  = $CargoHome
$env:RUSTUP_HOME = $RustupHome
$env:PATH        = "$CargoBin;$env:PATH"

$oldEAP = $ErrorActionPreference
$ErrorActionPreference = 'Continue'

if (Test-Path -LiteralPath $RustcExe) {
    $rv = & $RustcExe --version 2>&1
    Write-Log "rustc:  $rv"
}
if (Test-Path -LiteralPath $CargoExe) {
    $cv = & $CargoExe --version 2>&1
    Write-Log "cargo:  $cv"
}

$ErrorActionPreference = $oldEAP

# === Next steps ==============================================================

Write-Log '=== Bootstrap complete ==='

Write-Host ''
Write-Host '========================================' -ForegroundColor Green
Write-Host '  RUST BOOTSTRAP COMPLETE' -ForegroundColor Green
Write-Host '========================================' -ForegroundColor Green
Write-Host ''
Write-Host "  CARGO_HOME:  $CargoHome"
Write-Host "  RUSTUP_HOME: $RustupHome"
Write-Host "  Log file:    $LogFile"
Write-Host ''
Write-Host '  Next steps:' -ForegroundColor Cyan
Write-Host '    1. Open a NEW terminal (PATH was updated)'
Write-Host '    2. cd to the repository root'
Write-Host '    3. cargo build'
Write-Host '    4. cargo test'
Write-Host '    5. .\target\debug\ffwb.exe'
Write-Host ''
Write-Host '  For the full build/test/run workflow use:' -ForegroundColor Yellow
Write-Host '    powershell -File tools\powershell\ffwb_make.ps1'
Write-Host ''
