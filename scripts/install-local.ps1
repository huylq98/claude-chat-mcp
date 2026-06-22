# Register a built connector with your LOCAL Claude Desktop for review, by adding
# an entry to claude_desktop_config.json that points at the compiled binary and
# passes the connector's env vars.
#
# Until the GUI configurator ships, this is how you try a connector end-to-end.
#
# Usage:
#   ./scripts/install-local.ps1 atlassian @{ CONFLUENCE_URL="https://wiki.corp"; ATLASSIAN_TOKEN="pat" }
#   ./scripts/install-local.ps1 database  @{ DB_ENGINE="mysql"; DB_HOST="127.0.0.1"; DB_USER="root"; DB_PASSWORD="..." }
#
# Then fully quit and reopen Claude Desktop.
param(
    [Parameter(Mandatory = $true)][string]$Connector,
    [Parameter(Mandatory = $true)][hashtable]$Env,
    [switch]$Release
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$profile = if ($Release) { "release" } else { "debug" }
$exe = Join-Path $root "target\$profile\$Connector.exe"
if (-not (Test-Path $exe)) { throw "Binary not found: $exe. Build it first (./scripts/cargo.ps1 build)." }

$cfgPath = Join-Path $env:APPDATA "Claude\claude_desktop_config.json"
if (Test-Path $cfgPath) {
    $cfg = Get-Content $cfgPath -Raw | ConvertFrom-Json
} else {
    New-Item -ItemType Directory -Force -Path (Split-Path $cfgPath) | Out-Null
    $cfg = [pscustomobject]@{}
}
if (-not $cfg.PSObject.Properties['mcpServers']) {
    $cfg | Add-Member -NotePropertyName mcpServers -NotePropertyValue ([pscustomobject]@{})
}

$entry = [ordered]@{ command = $exe; env = $Env }
# Key the server entry as "claude-chat-mcp-<connector>" so it's easy to spot/remove.
$key = "claude-chat-mcp-$Connector"
$cfg.mcpServers | Add-Member -NotePropertyName $key -NotePropertyValue $entry -Force

$cfg | ConvertTo-Json -Depth 12 | Set-Content -Path $cfgPath -Encoding utf8
Write-Output "Registered '$key' -> $exe"
Write-Output "Config: $cfgPath"
Write-Output "Now fully quit Claude Desktop (tray -> Exit) and reopen it."
