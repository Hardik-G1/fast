# fast-tools binary installer for PowerShell (no Rust/Cargo needed)
# Expects fast.exe in the same folder as this script
# Run: .\install-bin.ps1

$ErrorActionPreference = "Stop"
$marker = "# fast"

# ── 1. Copy binary to a folder on PATH ─────────────────────────────────────
$src = Join-Path $PSScriptRoot "fast.exe"
if (!(Test-Path $src)) {
    Write-Host "fast.exe not found next to this script." -ForegroundColor Red
    exit 1
}

$dest = Join-Path $env:USERPROFILE ".fast"
if (!(Test-Path $dest)) { New-Item -Path $dest -ItemType Directory -Force | Out-Null }
Copy-Item $src (Join-Path $dest "fast.exe") -Force
Write-Host "Binary copied to $dest\fast.exe" -ForegroundColor Green

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$dest*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$dest", "User")
    $env:Path = "$env:Path;$dest"
    Write-Host "Added $dest to PATH" -ForegroundColor Green
} else {
    Write-Host "$dest already in PATH" -ForegroundColor Green
}

# ── 2. Add shell functions to $PROFILE ───────────────────────────────────────
if (!(Test-Path $PROFILE)) {
    $profileDir = Split-Path $PROFILE -Parent
    if (!(Test-Path $profileDir)) {
        New-Item -Path $profileDir -ItemType Directory -Force | Out-Null
    }
    New-Item -Path $PROFILE -ItemType File -Force | Out-Null
}

$snippet = @'

# fast-tools
function fcd  { $d = (& fast); if ($d) { Set-Location $d.Trim() } }
function fh   { fast hist }
function ftop { fast top }
function f    { $cmd = (& fast alias run $args); if ($cmd) { Invoke-Expression $cmd.Trim() } else { Write-Host "Alias '$args' not found" } }
$__fast_orig_prompt = if (Test-Path Function:\prompt) { Get-Content Function:\prompt } else { $null }
function prompt {
    $__fast_ok = $?
    if ($__fast_ok) {
        $c = (Get-History -Count 1 -EA SilentlyContinue).CommandLine
        if ($c) { fast hist --add $c }
    }
    if ($__fast_orig_prompt) { & ([scriptblock]::Create($__fast_orig_prompt)) } else { "PS $($executionContext.SessionState.Path.CurrentLocation)> " }
}
# fast-tools-end
'@

$existing = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
if ($existing -and $existing.Contains($marker)) {
    $pattern = '(?s)' + [regex]::Escape($marker) + '.*?' + [regex]::Escape('# fast-tools-end')
    $updated = [regex]::Replace($existing, $pattern, '').Trim()
    Set-Content -Path $PROFILE -Value ($updated + "`n" + $snippet)
    Write-Host "Shell functions updated in $PROFILE" -ForegroundColor Green
} else {
    Add-Content -Path $PROFILE -Value $snippet
    Write-Host "Shell functions added to $PROFILE" -ForegroundColor Green
}

# ── 3. Done ───────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Done! Reload your profile with:" -ForegroundColor Cyan
Write-Host "  . `$PROFILE" -ForegroundColor White
Write-Host ""
Write-Host "Commands available after reload:" -ForegroundColor Cyan
Write-Host "  fcd              - file browser (cd on Enter)"
Write-Host "  fh               - history picker (Enter runs command)"
Write-Host "  ftop             - system monitor"
Write-Host "  f <alias>        - run a saved alias"
Write-Host "  fast alias add <name> <cmd>  - save an alias"
Write-Host "  fast alias list  - list aliases"
