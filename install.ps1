# Install the latest javetas release binary for Windows.
$ErrorActionPreference = "Stop"

$Repo = "YuriRiegaAlva/javetas"

if ($env:PROCESSOR_ARCHITECTURE -eq "AMD64") {
    $Target = "javetas-windows-x86_64"
} else {
    Write-Error "javetas: unsupported architecture: $env:PROCESSOR_ARCHITECTURE"
    exit 1
}

$Url = "https://github.com/$Repo/releases/latest/download/$Target.zip"
$InstallDir = Join-Path $env:LOCALAPPDATA "javetas"
$Zip = Join-Path $env:TEMP "$Target.zip"

Write-Host "Downloading javetas ..."
Invoke-WebRequest -Uri $Url -OutFile $Zip

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Expand-Archive -Path $Zip -DestinationPath $InstallDir -Force
Remove-Item $Zip

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
}
$env:Path = "$InstallDir;$env:Path"

Write-Host "javetas installed to $InstallDir"
& (Join-Path $InstallDir "javetas.exe") version
