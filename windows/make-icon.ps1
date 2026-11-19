# Draws windows\DeskPet.ico (the .exe's icon) from Mochi's pixel art. Run once; the .ico is committed.
#
#   powershell -ExecutionPolicy Bypass -File windows\make-icon.ps1

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# Mochi with open eyes, cropped (same rows as the first pet in src/art.rs).
$rows = @(
    "..kk......kk..",
    ".kppk....kppk.",
    ".kpwkkkkkkwpk.",
    "kwwwwwwwwwwwwk",
    "kwwwwwwwwwwwwk",
    "kwwwwwwwkwwkwk",
    "kwwwwwwwkwwkwk",
    "kwwwwwwpwwwwpk",
    "kswwwwwwwwwwwk",
    "ksswwwwwwwwwwk",
    ".ksssswwwwwwk.",
    "..kkkkkkkkkk.."
)
$palette = @{
    'k' = [System.Drawing.Color]::FromArgb(51, 41, 61)
    'w' = [System.Drawing.Color]::FromArgb(252, 242, 227)
    's' = [System.Drawing.Color]::FromArgb(227, 209, 194)
    'p' = [System.Drawing.Color]::FromArgb(255, 158, 179)
}

$sizes = 16, 20, 24, 32, 40, 48, 64, 256
$images = foreach ($size in $sizes) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $cols = $rows[0].Length
    $fit = $size / [math]::Max($cols, $rows.Count)
    $k = if ([math]::Floor($fit) -ge 1 -and [math]::Floor($fit) / $fit -ge 0.8) { [math]::Floor($fit) } else { $fit }
    $ox = [math]::Round(($size - $cols * $k) / 2)
    $oy = [math]::Round(($size - $rows.Count * $k) / 2)
    for ($r = 0; $r -lt $rows.Count; $r++) {
        for ($c = 0; $c -lt $cols; $c++) {
            $ch = [string]$rows[$r][$c]
            if (-not $palette.ContainsKey($ch)) { continue }
            $x0 = [int][math]::Round($ox + $c * $k); $x1 = [int][math]::Round($ox + ($c + 1) * $k)
            $y0 = [int][math]::Round($oy + $r * $k); $y1 = [int][math]::Round($oy + ($r + 1) * $k)
            for ($y = $y0; $y -lt $y1; $y++) { for ($x = $x0; $x -lt $x1; $x++) { $bmp.SetPixel($x, $y, $palette[$ch]) } }
        }
    }
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    ,($ms.ToArray())
}

# ICO file: header, one directory entry per size, then the PNG data.
$out = New-Object System.IO.MemoryStream
$w = New-Object System.IO.BinaryWriter $out
$w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i]
    $w.Write([byte]($(if ($s -ge 256) { 0 } else { $s }))); $w.Write([byte]($(if ($s -ge 256) { 0 } else { $s })))
    $w.Write([byte]0); $w.Write([byte]0)
    $w.Write([uint16]1); $w.Write([uint16]32)
    $w.Write([uint32]$images[$i].Length); $w.Write([uint32]$offset)
    $offset += $images[$i].Length
}
foreach ($img in $images) { $w.Write($img) }
$w.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $PSScriptRoot 'DeskPet.ico'), $out.ToArray())
Write-Host "Wrote $(Join-Path $PSScriptRoot 'DeskPet.ico')"
