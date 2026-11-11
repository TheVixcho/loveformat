$ErrorActionPreference = "Stop"

$Repo = "TheVixcho/loveformat"
$BinName = "loveformat.exe"

if ((Test-Path "Cargo.toml") -and (Get-Command cargo -ErrorAction SilentlyContinue)) {
    cargo install --path .
    Write-Host "[OK] LoveFormat instalado exitosamente con cargo."
    exit 0
}

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\loveformat"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$Url = "https://github.com/$Repo/releases/latest/download/loveformat-windows-x86_64.zip"
$ZipPath = Join-Path $env:TEMP "loveformat.zip"

Write-Host ":: Descargando LoveFormat para Windows..."
Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
Remove-Item $ZipPath -Force

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

Write-Host "[OK] LoveFormat instalado correctamente en $InstallDir\$BinName"
Write-Host "Reinicia tu terminal o PowerShell y escribe 'loveformat' para comenzar."
