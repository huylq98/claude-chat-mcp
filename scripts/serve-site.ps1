# Serve the local connector catalog site.
# Copies the freshly-built registry.json into site/, then starts a static server.
# Usage: ./scripts/serve-site.ps1 [-Port 4321]
param([int]$Port = 4321)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$registry = Join-Path $root "registry.json"
$site = Join-Path $root "site"

if (-not (Test-Path $registry)) {
    Write-Warning "registry.json missing. Generating it (release build expected)..."
    & (Join-Path $PSScriptRoot "registry.ps1") --release
}
Copy-Item $registry (Join-Path $site "registry.json") -Force
Write-Output "Copied registry.json into site/"
Write-Output "Open http://localhost:$Port"
& node (Join-Path $PSScriptRoot "static-server.mjs") $site $Port
