# push-to-github.ps1
# Stages all changes, commits with a message, tags the phase, and pushes to GitHub.
# Usage:
#   .\push-to-github.ps1                          # uses default message
#   .\push-to-github.ps1 -Message "my message"   # custom commit message
#   .\push-to-github.ps1 -Tag "phase-ax"         # also creates a phase tag
#   .\push-to-github.ps1 -Message "Phase AX complete" -Tag "phase-ax"

param(
    [string]$Message = "chore: sync latest changes",
    [string]$Tag = ""
)

Set-Location $PSScriptRoot

Write-Host ""
Write-Host "=== FileForge Workbench — GitHub Push ===" -ForegroundColor Cyan
Write-Host ""

# 1. Show what will be committed
Write-Host "--- Changed files ---" -ForegroundColor Yellow
git status --short
Write-Host ""

# 2. Confirm before proceeding
$confirm = Read-Host "Proceed with commit and push? (y/n)"
if ($confirm -ne "y") {
    Write-Host "Aborted." -ForegroundColor Red
    exit 0
}

# 3. Stage all changes (including new files and deletions)
Write-Host ""
Write-Host "Staging all changes..." -ForegroundColor Yellow
git add -A
if ($LASTEXITCODE -ne 0) { Write-Host "git add failed." -ForegroundColor Red; exit 1 }

# 4. Commit
Write-Host "Committing: $Message" -ForegroundColor Yellow
git commit -m $Message
if ($LASTEXITCODE -ne 0) { Write-Host "git commit failed." -ForegroundColor Red; exit 1 }

# 5. Optional phase tag
if ($Tag -ne "") {
    Write-Host "Tagging: $Tag" -ForegroundColor Yellow
    git tag $Tag
    if ($LASTEXITCODE -ne 0) { Write-Host "git tag failed (tag may already exist)." -ForegroundColor Red; exit 1 }
}

# 6. Push commits
Write-Host "Pushing to origin/main..." -ForegroundColor Yellow
git push origin main
if ($LASTEXITCODE -ne 0) { Write-Host "git push failed." -ForegroundColor Red; exit 1 }

# 7. Push tags if one was created
if ($Tag -ne "") {
    Write-Host "Pushing tag $Tag..." -ForegroundColor Yellow
    git push origin $Tag
    if ($LASTEXITCODE -ne 0) { Write-Host "git push tag failed." -ForegroundColor Red; exit 1 }
}

Write-Host ""
Write-Host "Done. View at: https://github.com/AlanWynne/FileForgeWorkbench" -ForegroundColor Green
Write-Host ""
