# Compila o Glint (release) e instala em %LOCALAPPDATA%\Glint.
# Requisitos: Rust (MSVC), Node.js e WebView2 (já vem no Windows 11).
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$dest = Join-Path $env:LOCALAPPDATA 'Glint'
$exe  = Join-Path $dest 'glint.exe'

Write-Host '[1/4] Instalando dependências do Tauri CLI...'
Push-Location $root
if (-not (Test-Path 'node_modules\@tauri-apps\cli')) { npm install --no-audit --no-fund | Out-Null }

Write-Host '[2/4] Compilando (release)...'
npx tauri build --no-bundle
if ($LASTEXITCODE -ne 0) { throw 'Falha na compilação.' }
Pop-Location

Write-Host '[3/4] Instalando...'
Get-Process glint -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400
New-Item -ItemType Directory -Force $dest | Out-Null
Copy-Item (Join-Path $root 'src-tauri\target\release\glint.exe') $exe -Force

# na primeira execução o app liga "iniciar com o sistema" apontando para este exe
Remove-Item (Join-Path $dest 'first_run_done') -ErrorAction SilentlyContinue

Write-Host '[4/4] Iniciando...'
Start-Process $exe
Write-Host ''
Write-Host 'Pronto! Pressione Ctrl+Espaço em qualquer lugar. O ícone fica na bandeja do sistema.'
