$ErrorActionPreference = "Continue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$frontendDir = Join-Path $repoRoot "src\frontend"
$logPath = Join-Path $repoRoot ".frontend-web.log"

$env:npm_config_cache = Join-Path $repoRoot ".npm-cache"

Set-Location $frontendDir
$npm = if (Test-Path "M:\Dev\Lang\Node\npm.cmd") {
    "M:\Dev\Lang\Node\npm.cmd"
} else {
    "npm.cmd"
}
& $npm run dev *>&1 | Tee-Object -FilePath $logPath
