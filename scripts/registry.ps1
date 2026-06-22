# Aggregate every connector's --manifest output into a single registry.json.
# This is the single source of truth that the future configurator wizard and the
# website will both consume. Run after a build.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$profile = if ($args -contains "--release") { "release" } else { "debug" }
$binDir = Join-Path $root "target\$profile"

# Derive the connector list from the actual connector crates so the registry
# always matches what's built (db-core is a library, not under connectors/).
$connectors = Get-ChildItem (Join-Path $root "crates\connectors") -Directory | Select-Object -ExpandProperty Name | Sort-Object
$manifests = @()
foreach ($c in $connectors) {
    $exe = Join-Path $binDir "$c.exe"
    if (-not (Test-Path $exe)) { Write-Warning "missing $exe (build first)"; continue }
    $json = & $exe --manifest | Out-String
    $manifests += ($json | ConvertFrom-Json)
}
$registry = [ordered]@{ version = 1; connectors = $manifests }
$out = Join-Path $root "registry.json"
$registry | ConvertTo-Json -Depth 12 | Set-Content -Path $out -Encoding utf8
Write-Output "Wrote $out with $($manifests.Count) connectors."
