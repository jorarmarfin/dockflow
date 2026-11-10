# Compila DocFlow en Windows -> instalador NSIS (.exe). Salida: dist-release\
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
foreach ($c in 'node','npm','cargo') { if (-not (Get-Command $c -ErrorAction SilentlyContinue)) { throw "Falta '$c'. Ver README (Requisitos)." } }
if (-not (Test-Path node_modules)) { npm install }
Push-Location src-tauri; cargo test --release; if ($LASTEXITCODE) { throw 'Fallaron las pruebas' }; Pop-Location
npm run tauri build -- --bundles nsis
if ($LASTEXITCODE) { throw 'Falló el empaquetado' }
Remove-Item dist-release -Recurse -Force -ErrorAction SilentlyContinue
New-Item dist-release -ItemType Directory | Out-Null
Get-ChildItem src-tauri\target\release\bundle -Recurse -Filter *-setup.exe | Copy-Item -Destination dist-release
Get-ChildItem dist-release -File | Get-FileHash -Algorithm SHA256 | ForEach-Object { "$($_.Hash.ToLower())  $(Split-Path $_.Path -Leaf)" } | Set-Content dist-release\SHA256SUMS.txt
Get-ChildItem dist-release
