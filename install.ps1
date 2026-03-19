# fast-tools installer for PowerShell
# Run: irm https://... | iex   OR   .\install.ps1

$ErrorActionPreference = "Stop"
$marker = "# fast"


if (!(Get-Command cargo -EA SilentlyContinue)) {
    Write-Host "Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red; exit 1
}
if (!(Get-Command git -EA SilentlyContinue)) {
    Write-Host "Git not found. Install from https://git-scm.com" -ForegroundColor Red; exit 1
}

# ── 1. Build & install the binary ────────────────────────────────────────────
$repoUrl = "https://github.com/Hardik-G1/fast.git"
$srcDir = $PSScriptRoot
if (!$srcDir -or !(Test-Path (Join-Path $srcDir "Cargo.toml") -EA SilentlyContinue)) {
    # Running via pipe (irm | iex) — clone the repo to a temp dir
    Write-Host "Downloading source..." -ForegroundColor Cyan
    $tmp = Join-Path $env:TEMP "fast-tools-install"
    if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
    git clone --depth 1 $repoUrl $tmp
    if ($LASTEXITCODE -ne 0) { Write-Host "Git clone failed." -ForegroundColor Red; exit 1 }
    $srcDir = Join-Path $tmp "fast"
}
Write-Host "Building fast..." -ForegroundColor Cyan
cargo install --path $srcDir --quiet
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed." -ForegroundColor Red; exit 1 }
Write-Host "Binary installed." -ForegroundColor Green

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
