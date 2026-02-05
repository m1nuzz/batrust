---
name: Cargo Startup Validation
description: Validates Rust binary starts without runtime errors for long-running applications like daemons, tray apps, and servers. Checks compilation, runs unit tests, and validates startup behavior with timeout to prevent hanging on long-running processes.
---

# Cargo Startup Validation

## Purpose
Validates that `cargo run` starts successfully without runtime errors for long-running applications (daemons, tray apps, servers).

## Workflow

### 1. Run Unit Tests
```bash
cargo test --quiet
```

### 2. Check Compilation
```bash
cargo check --quiet
```

### 3. Validate Binary Startup (Critical)

For long-running applications on Windows, run this command directly in the console:

```powershell
Start-Process -FilePath "cargo" -ArgumentList "run" -RedirectStandardError "stderr.log" -RedirectStandardOutput "stdout.log" -PassThru | Wait-Process -Timeout 5 -ErrorAction SilentlyContinue

# If process is still running after timeout, kill it
$process = Get-Process | Where-Object {$_.ProcessName -like "*batrust*"} | Select-Object -First 1
if ($process -and !$process.HasExited) {
    Stop-Process -Id $process.Id -Force
}

# Combine output files
Get-Content stdout.log, stderr.log | Out-File -FilePath startup.log

# Check for errors
$errors = Select-String -Path startup.log -Pattern 'error|failed|panic|could not' -CaseSensitive
if ($errors) {
    Write-Host "❌ Runtime errors detected during startup"
    $errors | ForEach-Object { Write-Host $_ }
    Get-Content startup.log
    exit 1
} else {
    Write-Host "✅ No runtime errors detected during startup"
}

# Clean up temporary files
Remove-Item startup.log, stdout.log, stderr.log -ErrorAction SilentlyContinue
```

## Success Criteria
- ✅ Unit tests pass
- ✅ Code compiles
- ✅ Binary starts without errors in first 5 seconds
- ✅ No "Error", "Failed", "Panic", "Could not" in output during startup

## Failure Examples
```
Battery read error: Could not retrieve battery information from any feature
Warning: Could not create system tray
```
These are FAILURES even if exit code is 0.