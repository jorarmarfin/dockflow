# Levanta DocFlow en desarrollo (Windows). Uso: .\run.ps1 [web|test]
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
foreach ($c in 'node','npm','cargo') { if (-not (Get-Command $c -ErrorAction SilentlyContinue)) { throw "Falta '$c'. Ver README (Requisitos)." } }
if (-not (Test-Path node_modules)) { npm install }
switch ($args[0]) {
  'web'  { npm run dev }
  'test' { Push-Location src-tauri; cargo test --release; Pop-Location; npm test --if-present }
  default { npm run tauri dev }
}
