# Builds one pet and shows it on your taskbar, without installing it. Run it again after a change.
#
#   powershell -ExecutionPolicy Bypass -File windows\try-pet.ps1 bean            # just the pet
#   powershell -ExecutionPolicy Bypass -File windows\try-pet.ps1 bean -Tricks    # play every trick
#   powershell -ExecutionPolicy Bypass -File windows\try-pet.ps1 bean -Typing    # or the typing activity
#   powershell -ExecutionPolicy Bypass -File windows\try-pet.ps1 bean -Pastime   # or its pastime
#
# A running DeskPet is closed first (only one runs at a time).

param(
    [Parameter(Mandatory)] [string] $Id,
    [switch] $Tricks, [switch] $Typing, [switch] $Pastime, [switch] $Demo
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

& (Join-Path $PSScriptRoot 'build-pets.ps1') -Pets $Id
$exe = Join-Path $root 'build\DeskPet.exe'
if (-not (Test-Path $exe)) { & (Join-Path $PSScriptRoot 'build.ps1') }

Get-Process DeskPet -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300
$runArgs = @('--pets', (Join-Path $root 'build\pets'), '--pet', $Id)
if ($Tricks) { $runArgs += '--tricks' }
if ($Typing) { $runArgs += '--typing' }
if ($Pastime) { $runArgs += '--pastime' }
if ($Demo) { $runArgs += '--demo' }
Start-Process $exe -ArgumentList $runArgs
