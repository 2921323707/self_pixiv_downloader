$ErrorActionPreference = "Continue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$backendDir = Join-Path $repoRoot "src\backend"
$logPath = Join-Path $repoRoot ".backend-web.log"

if (-not $env:RUSTUP_HOME -and (Test-Path "M:\Dev\Config\.rustup")) {
    $env:RUSTUP_HOME = "M:\Dev\Config\.rustup"
}
if (-not $env:CARGO_HOME -and (Test-Path "M:\Dev\Config\.cargo")) {
    $env:CARGO_HOME = "M:\Dev\Config\.cargo"
}
$env:PIXIV_PLATFORM_BIND = if ($env:PIXIV_PLATFORM_BIND) {
    $env:PIXIV_PLATFORM_BIND
} else {
    "127.0.0.1:3000"
}

Set-Location $backendDir
$cargo = if ($env:CARGO_HOME -and (Test-Path (Join-Path $env:CARGO_HOME "bin\cargo.exe"))) {
    Join-Path $env:CARGO_HOME "bin\cargo.exe"
} else {
    "cargo.exe"
}
& $cargo run --bin server *>&1 | Tee-Object -FilePath $logPath
