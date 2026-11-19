# Builds DeskPet.exe for Windows 10/11 into build\DeskPet.exe.
#
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1           # build
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Run      # build, then start it (stops a running copy first)
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Zip      # also make build\DeskPet-<version>-windows.zip
#
# Needs Rust (rustup, stable, the x86_64-pc-windows-msvc toolchain) and the MSVC build tools.
# The exe links the C runtime statically, so it runs on any Windows 10/11 PC with nothing to install.

param([switch] $Run, [switch] $Zip, [string[]] $RunArgs = @())

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$version = if ($env:DESKPET_VERSION) { $env:DESKPET_VERSION } else { '1.2.0' }   # the release workflow passes the tag's version
$build = Join-Path $root 'build'
New-Item -ItemType Directory -Force $build | Out-Null

# The unlock-code secret lives in ~/.config/deskpet (created on first use), never in the repo.
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'Scripts\make-codes.ps1') -RustFile (Join-Path $build 'code_secret.rs')
if ($LASTEXITCODE -ne 0) { throw 'make-codes.ps1 failed' }

Get-Process DeskPet -ErrorAction SilentlyContinue | Stop-Process -Force
# cargo reads windows\.cargo\config.toml (static C runtime) only when run from this folder.
Push-Location $PSScriptRoot
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
} finally {
    Pop-Location
}

$exe = Join-Path $build 'DeskPet.exe'
Copy-Item (Join-Path $PSScriptRoot 'target\release\DeskPet.exe') $exe -Force
Write-Host ("Built $exe ({0:N0} KB)" -f ((Get-Item $exe).Length / 1KB))

if ($Zip) {
    $zipPath = Join-Path $build "DeskPet-$version-windows.zip"
    Compress-Archive -Path $exe -DestinationPath $zipPath -Force
    (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower() + "  DeskPet-$version-windows.zip" | Set-Content "$zipPath.sha256" -Encoding ascii
    Write-Host "Built $zipPath"
}

if ($Run) { Start-Process $exe -ArgumentList $RunArgs }
