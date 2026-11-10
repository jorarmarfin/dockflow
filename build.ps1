# Compila DocFlow en Windows -> instalador Inno Setup (.exe) con acceso directo en el escritorio.
# Requisitos: Node, Rust, Inno Setup 6 (https://jrsoftware.org/isdl.php). Salida: dist-release\
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
foreach ($c in 'node','npm','cargo') { if (-not (Get-Command $c -ErrorAction SilentlyContinue)) { throw "Falta '$c'. Ver README (Requisitos)." } }

$iscc = (Get-Command iscc -ErrorAction SilentlyContinue).Source
if (-not $iscc) {
  $iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $iscc) { throw 'Falta Inno Setup 6: https://jrsoftware.org/isdl.php' }

if (-not (Test-Path node_modules)) { npm install }
Push-Location src-tauri; cargo test --release; if ($LASTEXITCODE) { throw 'Fallaron las pruebas' }; Pop-Location

npm run tauri build -- --no-bundle
if ($LASTEXITCODE) { throw 'Falló la compilación' }

Remove-Item dist-release -Recurse -Force -ErrorAction SilentlyContinue
New-Item dist-release -ItemType Directory | Out-Null
$version = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version
& $iscc "/DAppVersion=$version" installer\docflow.iss
if ($LASTEXITCODE) { throw 'Falló Inno Setup' }

Get-ChildItem dist-release -File | Get-FileHash -Algorithm SHA256 | ForEach-Object { "$($_.Hash.ToLower())  $(Split-Path $_.Path -Leaf)" } | Set-Content dist-release\SHA256SUMS.txt
Get-ChildItem dist-release
