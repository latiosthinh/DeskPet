# Builds DeskPet.exe for Windows 10/11 into build\DeskPet.exe.
#
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1           # build
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Run      # build, then start it (stops a running copy first)
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1 -Zip      # also make build\DeskPet-<version>-windows.zip
#
# Needs the .NET SDK (any recent version; it fetches the .NET Framework 4.8 reference assemblies itself).

param([switch] $Run, [switch] $Zip, [string[]] $RunArgs = @())

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$version = '1.2.0'
$build = Join-Path $root 'build'
New-Item -ItemType Directory -Force $build | Out-Null

# The unlock-code secret lives in ~/.config/deskpet (created on first use), never in the repo.
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $root 'Scripts\make-codes.ps1') -CsFile (Join-Path $build 'CodeSecret.cs')
if ($LASTEXITCODE -ne 0) { throw 'make-codes.ps1 failed' }

Get-Process DeskPet -ErrorAction SilentlyContinue | Stop-Process -Force
dotnet build (Join-Path $PSScriptRoot 'DeskPet.csproj') -c Release -nologo -v quiet
if ($LASTEXITCODE -ne 0) { throw 'dotnet build failed' }

$exe = Join-Path $build 'DeskPet.exe'
Copy-Item (Join-Path $PSScriptRoot 'bin\Release\net48\DeskPet.exe') $exe -Force
Write-Host "Built $exe"

if ($Zip) {
    $zipPath = Join-Path $build "DeskPet-$version-windows.zip"
    Compress-Archive -Path $exe -DestinationPath $zipPath -Force
    (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower() + "  DeskPet-$version-windows.zip" | Set-Content "$zipPath.sha256" -Encoding ascii
    Write-Host "Built $zipPath"
}

if ($Run) { Start-Process $exe -ArgumentList $RunArgs }
