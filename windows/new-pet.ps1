# Starts a new pet from pets\_template: a body, colours and two example tricks to change.
#
#   powershell -ExecutionPolicy Bypass -File windows\new-pet.ps1 bean "Bean the blob" -Author "Your Name"
#
# Then: windows\try-pet.ps1 bean -Tricks   (see it)   and   windows\check-pet.ps1 bean   (check it).

param(
    [Parameter(Mandatory)] [string] $Id,
    [Parameter(Mandatory)] [string] $Name,
    [string] $Author = $env:USERNAME
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

if ($Id -notmatch '^[a-z0-9][a-z0-9-]{0,31}$') { throw "The id is lowercase letters, digits and -, like ""bean"" or ""moss-cat""." }
if ($Name.Length -gt 40) { throw 'Keep the name to 40 characters.' }
$dest = Join-Path $root "pets\$Id"
if (Test-Path $dest) { throw "pets\$Id already exists." }

Copy-Item (Join-Path $root 'pets\_template') $dest -Recurse
$quote = { param($s) $s.Replace('\', '\\').Replace('"', '\"') }
Get-ChildItem $dest -Recurse -File | ForEach-Object {
    $text = [IO.File]::ReadAllText($_.FullName)
    $text = $text.Replace('__ID__', $Id).Replace('__NAME__', (& $quote $Name)).Replace('__AUTHOR__', (& $quote $Author))
    [IO.File]::WriteAllText($_.FullName, $text)
}
Write-Host "Made pets\$Id. Building it..."
& (Join-Path $PSScriptRoot 'check-pet.ps1') $Id
if ($LASTEXITCODE -ne 0) { Write-Host "pets\$Id was made, but the check found problems (above)."; exit 1 }
Write-Host ""
Write-Host "Next:  powershell -ExecutionPolicy Bypass -File windows\try-pet.ps1 $Id -Tricks"
