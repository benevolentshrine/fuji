Write-Host "===================================================" -ForegroundColor Cyan
Write-Host "    VANGUARD - Codebase Intelligence Engine        " -ForegroundColor Cyan
Write-Host "===================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Bypassing Windows OS File Locks by using safe temp directory..." -ForegroundColor Yellow

$env:CARGO_TARGET_DIR="C:\temp\vanguard_build"
cargo run --release

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "[ERROR] Vanguard engine closed unexpectedly." -ForegroundColor Red
    Pause
}
