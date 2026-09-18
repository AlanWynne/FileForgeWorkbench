#requires -Version 7.0
<#
.SYNOPSIS
Finds Cargo workspace crates containing egui UI code and adds egui_kittest as a dev dependency.

.DESCRIPTION
Uses `cargo metadata --no-deps` to enumerate workspace packages, then scores each crate using:
  * Cargo dependencies such as egui, eframe, egui_extras, and egui-wgpu
  * Rust source references such as egui::, eframe::, impl eframe::App, and CentralPanel

By default, matching crates are modified. Use -WhatIf to preview safely.
Existing egui_kittest dependencies are left unchanged.

.EXAMPLE
./Add-EguiKittest.ps1 -WhatIf -Verbose

.EXAMPLE
./Add-EguiKittest.ps1

.EXAMPLE
./Add-EguiKittest.ps1 -MinimumScore 4 -Version 0.36
#>
[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'Medium')]
param(
    [Parameter()]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Container })]
    [string] $WorkspaceRoot = (Get-Location).Path,

    [Parameter()]
    [ValidateRange(1, 100)]
    [int] $MinimumScore = 3,

    [Parameter()]
    [ValidatePattern('^\d+\.\d+(?:\.\d+)?(?:[-+][0-9A-Za-z.-]+)?$')]
    [string] $Version,

    [Parameter()]
    [switch] $IncludeExamples,

    [Parameter()]
    [switch] $PassThru
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-CargoMetadata {
    param([string] $Root)

    $output = & cargo metadata --format-version 1 --no-deps --manifest-path (Join-Path $Root 'Cargo.toml') 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed:`n$($output -join [Environment]::NewLine)"
    }
    return ($output -join [Environment]::NewLine) | ConvertFrom-Json -Depth 100
}

function Get-EguiEvidence {
    param($Package, [switch] $ScanExamples)

    $manifestPath = [IO.Path]::GetFullPath($Package.manifest_path)
    $crateRoot = Split-Path -Parent $manifestPath
    $evidence = [Collections.Generic.List[string]]::new()
    $score = 0

    $dependencyNames = @($Package.dependencies | ForEach-Object { $_.name })
    foreach ($name in $dependencyNames) {
        switch -Regex ($name) {
            '^egui$'          { $score += 5; $evidence.Add('Cargo dependency: egui'); break }
            '^eframe$'        { $score += 5; $evidence.Add('Cargo dependency: eframe'); break }
            '^egui_'          { $score += 2; $evidence.Add("Cargo dependency: $name"); break }
        }
    }

    $roots = [Collections.Generic.List[string]]::new()
    foreach ($relative in @('src', 'tests')) {
        $candidate = Join-Path $crateRoot $relative
        if (Test-Path -LiteralPath $candidate -PathType Container) { $roots.Add($candidate) }
    }
    if ($ScanExamples) {
        $candidate = Join-Path $crateRoot 'examples'
        if (Test-Path -LiteralPath $candidate -PathType Container) { $roots.Add($candidate) }
    }

    $patterns = [ordered]@{
        'impl\s+(?:eframe::)?App\s+for' = @{ Score = 5; Label = 'eframe App implementation' }
        'eframe::run_native|run_native\s*\(' = @{ Score = 4; Label = 'native egui application entry point' }
        '\begui::'                       = @{ Score = 3; Label = 'egui API usage' }
        '\beframe::'                     = @{ Score = 3; Label = 'eframe API usage' }
        '\b(?:CentralPanel|SidePanel|TopBottomPanel|Window)::' = @{ Score = 2; Label = 'egui panel/window usage' }
    }

    $files = foreach ($root in $roots) {
        Get-ChildItem -LiteralPath $root -Filter '*.rs' -File -Recurse -ErrorAction SilentlyContinue
    }

    foreach ($entry in $patterns.GetEnumerator()) {
        $match = $files | Select-String -Pattern $entry.Key -List -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($match) {
            $score += $entry.Value.Score
            $relative = [IO.Path]::GetRelativePath($crateRoot, $match.Path)
            $evidence.Add("$($entry.Value.Label): $relative")
        }
    }

    $existing = $dependencyNames -contains 'egui_kittest'
    $eguiDependency = $Package.dependencies | Where-Object name -eq 'egui' | Select-Object -First 1

    [pscustomobject]@{
        Package      = $Package.name
        ManifestPath = $manifestPath
        Score        = $score
        Relevant     = $score -ge $MinimumScore
        Existing     = $existing
        EguiVersion  = if ($eguiDependency) { $eguiDependency.req } else { $null }
        Evidence     = $evidence -join '; '
        Status       = if ($existing) { 'Already present' } elseif ($score -ge $MinimumScore) { 'Candidate' } else { 'Skipped' }
    }
}

$WorkspaceRoot = (Resolve-Path -LiteralPath $WorkspaceRoot).Path
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'cargo was not found on PATH.'
}

$metadata = Invoke-CargoMetadata -Root $WorkspaceRoot
$workspaceIds = [Collections.Generic.HashSet[string]]::new([string[]]$metadata.workspace_members)
$packages = @($metadata.packages | Where-Object { $workspaceIds.Contains([string]$_.id) })

$results = foreach ($package in $packages) {
    Get-EguiEvidence -Package $package -ScanExamples:$IncludeExamples
}

$candidates = @($results | Where-Object Relevant | Sort-Object Package)
if ($candidates.Count -eq 0) {
    Write-Warning "No crate reached the relevance threshold of $MinimumScore."
    $results | Sort-Object Score -Descending | Format-Table Package, Score, Status, Evidence -AutoSize
    return
}

Write-Host "Detected $($candidates.Count) egui-related crate(s):" -ForegroundColor Cyan
$candidates | Format-Table Package, Score, Existing, EguiVersion, Evidence -AutoSize -Wrap

foreach ($candidate in $candidates) {
    if ($candidate.Existing) {
        Write-Host "Skipping $($candidate.Package): egui_kittest already exists." -ForegroundColor DarkGray
        continue
    }

    $dependency = if ($Version) { "egui_kittest@$Version" } else { 'egui_kittest' }
    $action = "cargo add --dev $dependency"

    if ($PSCmdlet.ShouldProcess($candidate.ManifestPath, $action)) {
        $cargoOutput = & cargo add --manifest-path $candidate.ManifestPath --dev $dependency 2>&1
        if ($LASTEXITCODE -ne 0) {
            $candidate.Status = 'Failed'
            Write-Error "Failed for $($candidate.Package):`n$($cargoOutput -join [Environment]::NewLine)" -ErrorAction Continue
        }
        else {
            $candidate.Status = 'Added'
            Write-Host "Added egui_kittest to $($candidate.Package)." -ForegroundColor Green
        }
    }
    elseif ($WhatIfPreference) {
        $candidate.Status = 'Would add'
    }
}

Write-Host "`nSummary:" -ForegroundColor Cyan
$candidates | Format-Table Package, Score, Status, ManifestPath -AutoSize

if ($PassThru) { $results }
