# Builds Mai, puts assets/ next to the executable, and launches it.
# Usage: scripts/run.ps1 [-Dev]   (-Dev uses the debug build instead of release)
param([switch]$Dev)
$ErrorActionPreference = 'Stop'

$root = Split-Path $PSScriptRoot -Parent
$profileDir = if ($Dev) { 'debug' } else { 'release' }
$out = Join-Path $root "target\$profileDir"

# Only one instance may run, so a leftover one would make the new launch exit silently.
Get-Process mai -ErrorAction SilentlyContinue | Stop-Process -Force

Push-Location $root
try {
    if ($Dev) { cargo build } else { cargo build --release }
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

# Replace rather than merge, so assets deleted from the repo don't linger next to the exe.
Remove-Item (Join-Path $out 'assets') -Recurse -Force -ErrorAction SilentlyContinue
Copy-Item (Join-Path $root 'assets') $out -Recurse

Start-Process (Join-Path $out 'mai.exe')
