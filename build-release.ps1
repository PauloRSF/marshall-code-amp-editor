# Build a standalone Marshall CODE Editor and copy it to the Desktop (Windows).
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "Building release..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$exeName = "MarshallCodeEditor.exe"
$src = Join-Path $PSScriptRoot "target\release\$exeName"
if (-not (Test-Path $src)) {
    Write-Error "Release binary not found: $src"
}

$desktop = [Environment]::GetFolderPath("Desktop")
$destName = "Marshall CODE Editor.exe"
$dest = Join-Path $desktop $destName
Copy-Item -Path $src -Destination $dest -Force

$sizeMb = [math]::Round((Get-Item $dest).Length / 1MB, 1)
Write-Host ""
Write-Host "Done." -ForegroundColor Green
Write-Host "  Built:   $src"
Write-Host "  Desktop: $dest  ($sizeMb MB)"
