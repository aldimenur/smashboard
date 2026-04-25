param(
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not $SkipBuild) {
  Write-Host "Building portable executable (no installer)..." -ForegroundColor Cyan
  & npm run tauri build -- --bundles none
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }
}

$exePath = Join-Path $repoRoot "src-tauri\target\release\sfx-board.exe"
if (-not (Test-Path $exePath)) {
  Write-Host "Portable executable not found: $exePath" -ForegroundColor Red
  Write-Host "Run: npm run build:portable"
  exit 1
}

$distPath = Join-Path $repoRoot "dist"
if (-not (Test-Path $distPath)) {
  Write-Host "Frontend dist folder not found: $distPath" -ForegroundColor Red
  Write-Host "Run: npm run build"
  exit 1
}

$portableRoot = Join-Path $repoRoot "artifacts\portable"
$portableDir = Join-Path $portableRoot "SFX-Board-Portable"
$zipPath = Join-Path $portableRoot "SFX-Board-Portable.zip"

New-Item -Path $portableRoot -ItemType Directory -Force | Out-Null
if (Test-Path $portableDir) {
  Remove-Item -LiteralPath $portableDir -Recurse -Force
}
if (Test-Path $zipPath) {
  Remove-Item -LiteralPath $zipPath -Force
}

New-Item -Path $portableDir -ItemType Directory -Force | Out-Null
Copy-Item -LiteralPath $exePath -Destination (Join-Path $portableDir "sfx-board.exe")
Copy-Item -LiteralPath $distPath -Destination (Join-Path $portableDir "dist") -Recurse

@"
SFX Board Portable

1. Ensure Microsoft Edge WebView2 Runtime is installed.
2. Run sfx-board.exe.
3. Keep the dist folder next to sfx-board.exe.
"@ | Set-Content -LiteralPath (Join-Path $portableDir "README-portable.txt")

if (Get-Command tar -ErrorAction SilentlyContinue) {
  & tar -a -c -f $zipPath -C $portableDir .
  if ($LASTEXITCODE -ne 0) {
    Write-Host "tar zip failed, falling back to Compress-Archive..." -ForegroundColor Yellow
    Compress-Archive -Path (Join-Path $portableDir "*") -DestinationPath $zipPath
  }
} else {
  Compress-Archive -Path (Join-Path $portableDir "*") -DestinationPath $zipPath
}

Write-Host "`nPortable artifacts ready:" -ForegroundColor Green
Write-Host (" - {0}" -f $portableDir)
Write-Host (" - {0}" -f $zipPath)
