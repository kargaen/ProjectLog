$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$releaseLogPath = Join-Path $repoRoot "RELEASE_LOG.md"
$releaseLogTemplate = @(
    "# Release Log",
    "",
    "Write the next release message here as you build features.",
    "",
    "Guidelines:",
    "- Keep the first line short. Git will use it as the commit subject.",
    "- Add extra lines below for details when useful.",
    "- Remove placeholder text before releasing."
)

function Get-JsonVersion {
    param([string]$Path)

    return (Get-Content $Path -Raw | ConvertFrom-Json).version
}

function Set-JsonVersion {
    param(
        [string]$Path,
        [string]$Version
    )

    $json = Get-Content $Path -Raw | ConvertFrom-Json
    $json.version = $Version

    if ($Path -like "*package-lock.json" -and $json.packages) {
        $rootPackage = $json.packages.PSObject.Properties[""]
        if ($rootPackage) {
            $rootPackage.Value.version = $Version
        }
    }

    $json | ConvertTo-Json -Depth 100 | Set-Content $Path -Encoding utf8
}

function Update-TextVersion {
    param(
        [string]$Path,
        [string]$Pattern,
        [string]$Replacement
    )

    $content = Get-Content $Path -Raw
    $updated = [regex]::Replace($content, $Pattern, $Replacement, 1)

    if ($updated -eq $content) {
        throw "Could not update version in $Path."
    }

    Set-Content $Path $updated -Encoding utf8
}

function Reset-ReleaseLog {
    Set-Content $releaseLogPath ($releaseLogTemplate -join "`r`n") -Encoding utf8
}

function Get-ReleaseMessage {
    if (-not (Test-Path $releaseLogPath)) {
        throw "RELEASE_LOG.md was not found."
    }

    $raw = Get-Content $releaseLogPath -Raw
    $normalized = $raw -replace "`r", ""
    $lines = $normalized -split "`n"
    $messageLines = @()

    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if (-not $trimmed) {
            if ($messageLines.Count -gt 0 -and $messageLines[-1] -ne "") {
                $messageLines += ""
            }
            continue
        }

        if ($trimmed -eq "# Release Log") { continue }
        if ($trimmed -eq "Guidelines:") { break }
        if ($trimmed -eq "Write the next release message here as you build features.") { continue }
        if ($trimmed -like "- Keep the first line short*") { break }
        if ($trimmed -like "- Add extra lines below*") { break }
        if ($trimmed -like "- Remove placeholder text before releasing.*") { break }

        $messageLines += $line.TrimEnd()
    }

    while ($messageLines.Count -gt 0 -and $messageLines[-1] -eq "") {
        $messageLines = $messageLines[0..($messageLines.Count - 2)]
    }

    $message = ($messageLines -join "`r`n").Trim()
    if (-not $message) {
        throw "RELEASE_LOG.md is empty. Add the release message before creating a new version."
    }

    return $message
}

function Update-CargoLockVersion {
    param(
        [string]$Path,
        [string]$Version
    )

    $content = Get-Content $Path -Raw
    $pattern = '(?s)(name = "project-log"\r?\nversion = ").*?(")'
    $updated = [regex]::Replace(
        $content,
        $pattern,
        {
            param($match)
            return $match.Groups[1].Value + $Version + $match.Groups[2].Value
        },
        1
    )

    if ($updated -eq $content) {
        throw "Could not update version in $Path."
    }

    Set-Content $Path $updated -Encoding utf8
}

$currentVersion = Get-JsonVersion "package.json"
Write-Host "Current version: $currentVersion"
Write-Host "This will update version files, create a release commit, tag v<version>, and push both branch and tag."

$newVersion = Read-Host "Enter new version number"
$newVersion = $newVersion.Trim()

if (-not $newVersion) {
    Write-Host "No version entered. Release cancelled."
    exit 0
}

if ($newVersion -eq $currentVersion) {
    throw "New version must differ from the current version."
}

if ($newVersion -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') {
    throw "Version must look like 1.2.3 or 1.2.3-beta.1"
}

$confirmation = Read-Host "Type the version again to confirm release"
if ($confirmation.Trim() -ne $newVersion) {
    Write-Host "Confirmation did not match. Release cancelled."
    exit 0
}

$existingTag = git tag -l "v$newVersion"
if ($existingTag) {
    throw "Tag v$newVersion already exists."
}

$releaseMessage = Get-ReleaseMessage
$commitMessageFile = [System.IO.Path]::GetTempFileName()
try {
    Set-Content $commitMessageFile $releaseMessage -Encoding utf8

    Set-JsonVersion "package.json" $newVersion
    Set-JsonVersion "package-lock.json" $newVersion
    Set-JsonVersion "src-tauri/tauri.conf.json" $newVersion
    Update-TextVersion "src-tauri/Cargo.toml" '^version = ".*?"' "version = `"$newVersion`""
    Update-CargoLockVersion "src-tauri/Cargo.lock" $newVersion
    Reset-ReleaseLog

    git add -A
    git commit -F $commitMessageFile
    git tag "v$newVersion"

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

    git push origin "v$newVersion"

    Write-Host "Released version $newVersion on branch $branch"
}
finally {
    Remove-Item $commitMessageFile -ErrorAction SilentlyContinue
}
