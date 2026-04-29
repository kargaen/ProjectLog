$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$branch = (git branch --show-current).Trim()
if (-not $branch) {
    throw "Could not determine the current branch."
}

$upstream = git rev-parse --abbrev-ref --symbolic-full-name "@{u}" 2>$null
if ($LASTEXITCODE -eq 0 -and $upstream) {
    git push
}
else {
    git push -u origin $branch
}

Write-Host "Pushed branch: $branch"
