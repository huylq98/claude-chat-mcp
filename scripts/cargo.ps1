# Wrapper that loads the MSVC dev environment, then runs cargo with the given args.
# The MSVC toolchain (link.exe + LIB/INCLUDE paths) is required to build on Windows
# but is not on PATH by default. Usage:  ./scripts/cargo.ps1 build --release
$ErrorActionPreference = "Stop"

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere)) { throw "vswhere not found; install Visual Studio Build Tools with the C++ workload." }
$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vsPath) { throw "No VS install with the C++ (VC Tools) component found." }

Import-Module (Join-Path $vsPath "Common7\Tools\Microsoft.VisualStudio.DevShell.dll")
Enter-VsDevShell -VsInstallPath $vsPath -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64' | Out-Null

Set-Location $PSScriptRoot\..
& cargo @args
exit $LASTEXITCODE
