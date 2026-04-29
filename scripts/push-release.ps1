$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

node scripts/release.mjs @args
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
