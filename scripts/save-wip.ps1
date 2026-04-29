$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$statusLines = git status --porcelain
if (-not $statusLines) {
    Write-Host "Working tree is clean. No WIP commit created."
    exit 0
}

git add -A

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm"
$message = "WIP $timestamp"

git commit -m $message

Write-Host "Created commit: $message"
