# Builds pet modules (<id>.deskpet, WebAssembly) into build\pets.
#
#   powershell -ExecutionPolicy Bypass -File windows\build-pets.ps1                    # every pet in pets\
#   powershell -ExecutionPolicy Bypass -File windows\build-pets.ps1 -Pets ninja,moona  # just these
#
# Try them in the app with:  build\DeskPet.exe --pets build\pets --pet <id>

param([string[]] $Pets = @())

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$target = 'wasm32-unknown-unknown'

if (-not $Pets) {
    $Pets = Get-ChildItem (Join-Path $root 'pets') -Directory | Where-Object { $_.Name -notlike '_*' } | ForEach-Object Name
}
if (-not ((rustup target list --installed) -contains $target)) {
    rustup target add $target
    if ($LASTEXITCODE -ne 0) { throw "couldn't add the $target Rust target" }
}

$out = Join-Path $root 'build\pets'
New-Item -ItemType Directory -Force $out | Out-Null
Push-Location $root
try {
    $cargoArgs = @('build', '--release', '--target', $target) + ($Pets | ForEach-Object { '-p', "pet-$_" })
    cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
} finally {
    Pop-Location
}

foreach ($p in $Pets) {
    $wasm = Join-Path $root "target\$target\release\pet_$($p -replace '-', '_').wasm"
    $file = Join-Path $out "$p.deskpet"
    Copy-Item $wasm $file -Force
    Write-Host ("Built $file ({0:N0} KB)" -f ((Get-Item $file).Length / 1KB))
}
