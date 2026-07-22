# Publish plotine workspace crates to crates.io in dependency order.
# Usage:
#   .\scripts\publish.ps1 -DryRun
#   .\scripts\publish.ps1

param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

# Dependencies before dependents. plotine-pyplot is last (depends on plotine).
$crates = @(
    "plotine-core",
    "plotine-render",
    "plotine-text",
    "plotine-backend-skia",
    "plotine-backend-svg",
    "plotine-backend-pdf",
    "plotine-backend-pgf",
    "plotine",
    "plotine-pyplot"
)

$extra = @()
if ($DryRun) {
    $extra += "--dry-run"
    $extra += "--allow-dirty"
    Write-Host "Dry run: cargo publish --dry-run for each crate" -ForegroundColor Cyan
    Write-Host "Note: dependents fail dry-run until prior crates exist on crates.io;" -ForegroundColor DarkGray
    Write-Host "      use 'cargo package -p CRATE --allow-dirty --no-verify' for local packaging checks." -ForegroundColor DarkGray
}

foreach ($crate in $crates) {
    Write-Host ""
    Write-Host "=== Publishing $crate ===" -ForegroundColor Green
    # Explicit registry: some mirrors replace crates-io and block publish otherwise.
    cargo publish -p $crate --registry crates-io @extra
    if (-not $DryRun) {
        Write-Host "Waiting 45s for crates.io index..." -ForegroundColor DarkGray
        Start-Sleep -Seconds 45
    }
}

Write-Host ""
Write-Host "Done." -ForegroundColor Green
