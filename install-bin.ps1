# fast-tools binary installer for PowerShell (no Rust/Cargo needed)
# Expects fast.exe in the same folder as this script
# Run: .\install-bin.ps1

$ErrorActionPreference = "Stop"

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

# ── 2. Write shell functions to ~/.fast/init.ps1 ─────────────────────────────
$initFile = Join-Path $dest "init.ps1"
$snippet = @'
# fast-tools shell functions
function fcd  { $d = (& fast cd); if ($d) { Set-Location $d.Trim() } }
function fh   { fast hist }
function ftop { fast top }
function f    { $cmd = (& fast alias run $args); if ($cmd) { Invoke-Expression $cmd.Trim() } else { Write-Host "Alias '$args' not found" } }
# Record last command to fast history (runs in background to avoid slowing prompt)
$__fast_last_hist_id = 0
function __fast_hist_record {
    $last = Get-History -Count 1 -EA SilentlyContinue
    if ($last -and $last.Id -ne $script:__fast_last_hist_id) {
        $script:__fast_last_hist_id = $last.Id
        Start-Process -FilePath "fast" -ArgumentList "hist","--add",$last.CommandLine -WindowStyle Hidden -EA SilentlyContinue
    }
}
Register-EngineEvent -SourceIdentifier PowerShell.OnIdle -Action { __fast_hist_record } -EA SilentlyContinue | Out-Null
'@
Set-Content -Path $initFile -Value $snippet -Force
Write-Host "Shell functions written to $initFile" -ForegroundColor Green

# ── 3. Add one-liner source to $PROFILE ───────────────────────────────────────
$sourceLine = ". `"$initFile`""
try {
    if (!(Test-Path $PROFILE)) {
        $profileDir = Split-Path $PROFILE -Parent
        [System.IO.Directory]::CreateDirectory($profileDir) | Out-Null
        [System.IO.File]::WriteAllText($PROFILE, "$sourceLine`n")
        Write-Host "Created $PROFILE with fast-tools loader" -ForegroundColor Green
    } else {
        $existing = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
        # Remove old inline fast-tools block if present
        if ($existing -and $existing.Contains("# fast-tools")) {
            $pattern = '(?s)# fast-tools.*?# fast-tools-end\r?\n?'
            $existing = [regex]::Replace($existing, $pattern, '').Trim()
            Set-Content -Path $PROFILE -Value $existing
        }
        # Add source line if not already there
        $existing = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
        if (!$existing -or !$existing.Contains($initFile)) {
            Add-Content -Path $PROFILE -Value "`n$sourceLine"
            Write-Host "Added fast-tools loader to $PROFILE" -ForegroundColor Green
        } else {
            Write-Host "fast-tools loader already in $PROFILE" -ForegroundColor Green
        }
    }
} catch {
    Write-Host "Could not update $PROFILE" -ForegroundColor Yellow
    Write-Host "Add this line to your PowerShell profile manually:" -ForegroundColor Yellow
    Write-Host "  $sourceLine" -ForegroundColor White
}

# ── 4. Done ───────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Done! Restart PowerShell or run:" -ForegroundColor Cyan
Write-Host "  $sourceLine" -ForegroundColor White
Write-Host ""
Write-Host "Commands available:" -ForegroundColor Cyan
Write-Host "  fcd              - file browser (cd on Enter)"
Write-Host "  fh               - history picker (Enter runs command)"
Write-Host "  ftop             - system monitor"
Write-Host "  f <alias>        - run a saved alias"
Write-Host "  fast alias add <name> <cmd>  - save an alias"
Write-Host "  fast alias list  - list aliases"
