# Automated test runner

Write-Host "🧪 Running traybattery test suite..." -ForegroundColor Cyan

# 1. Unit tests
Write-Host "`n=== Unit Tests ===" -ForegroundColor Yellow
cargo test --lib

# 2. Integration tests (потребують реальний пристрій)
Write-Host "`n=== Integration Tests (requires device) ===" -ForegroundColor Yellow
Write-Host "Run manually: cargo test --test device_integration_tests -- --ignored" -ForegroundColor Gray

# 3. Visual tests
Write-Host "`n=== Visual Tests ===" -ForegroundColor Yellow
cargo test tray_visual_tests

# 4. Battery parsing tests
Write-Host "`n=== Battery Parsing Tests ===" -ForegroundColor Yellow
cargo test battery_parsing_tests

# 5. Battery validation tests
Write-Host "`n=== Battery Validation Tests ===" -ForegroundColor Yellow
cargo test battery_validation_tests

# 6. Build release
Write-Host "`n=== Build Release ===" -ForegroundColor Yellow
cargo build --release

Write-Host "`n✅ Test suite completed!" -ForegroundColor Green
Write-Host "📋 Manual tests checklist: See TESTING.md" -ForegroundColor Cyan