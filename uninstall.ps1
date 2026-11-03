# Remove o Glint (processo, arquivos, inicialização automática e dados locais).
Get-Process glint -ErrorAction SilentlyContinue | Stop-Process -Force
$run = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
foreach ($n in 'Glint') {
    if (Get-ItemProperty $run -Name $n -ErrorAction SilentlyContinue) { Remove-ItemProperty $run -Name $n }
}
Remove-Item (Join-Path $env:LOCALAPPDATA 'Glint') -Recurse -Force -ErrorAction SilentlyContinue
Write-Host 'Glint removido.'
