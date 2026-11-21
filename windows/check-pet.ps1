# Builds pets and checks them with petshot: every trick, activity and reminder is played off screen,
# and anything wrong is reported (art rows of the wrong length, colours missing from the palette,
# a trick that crashes or runs too long, a missing description...).
#
#   powershell -ExecutionPolicy Bypass -File windows\check-pet.ps1 bean          # one pet
#   powershell -ExecutionPolicy Bypass -File windows\check-pet.ps1               # every pet
#   powershell -ExecutionPolicy Bypass -File windows\check-pet.ps1 bean -Gifs    # also make its GIFs in build\gifs\bean

param([string[]] $Pets = @(), [switch] $Gifs)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

& (Join-Path $PSScriptRoot 'build-pets.ps1') -Pets $Pets
if (-not $Pets) {
    $Pets = Get-ChildItem (Join-Path $root 'pets') -Directory | Where-Object { $_.Name -notlike '_*' } | ForEach-Object Name
}

Push-Location $root
try {
    $ErrorActionPreference = 'Continue'
    cargo build --release -p deskpet --bin petshot
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -ne 0) { throw 'building petshot failed' }
} finally {
    Pop-Location
}
$petshot = Join-Path $root 'target\release\petshot.exe'

$files = @($Pets | ForEach-Object { Join-Path $root "build\pets\$_.deskpet" })
& $petshot check @files
if ($LASTEXITCODE -ne 0) { exit 1 }

if ($Gifs) {
    foreach ($p in $Pets) {
        & $petshot gifs (Join-Path $root "build\pets\$p.deskpet") (Join-Path $root "build\gifs\$p")
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
}
