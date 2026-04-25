param(
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not $SkipBuild) {
  Write-Host "Building desktop app bundles (NSIS + MSI)..." -ForegroundColor Cyan
  & npm run tauri build
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }
}

$bundleDir = Join-Path $repoRoot "src-tauri\target\release\bundle"
if (-not (Test-Path $bundleDir)) {
  Write-Host "Bundle directory not found yet: $bundleDir" -ForegroundColor Yellow
  Write-Host "Run: npm run build:app"
  exit 0
}

$artifacts = Get-ChildItem -Path $bundleDir -Recurse -File -Include *.exe, *.msi |
  Sort-Object LastWriteTime -Descending

if (-not $artifacts) {
  Write-Host "No installer artifacts found under: $bundleDir" -ForegroundColor Yellow
  Write-Host "Run: npm run build:app"
  exit 0
}

Write-Host "`nReady installers:" -ForegroundColor Green
foreach ($artifact in $artifacts) {
  Write-Host (" - {0}" -f $artifact.FullName)
}
