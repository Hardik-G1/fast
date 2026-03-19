#!/usr/bin/env bash
# fast-tools installer for Bash/Zsh
# Run: bash install.sh

set -e
MARKER="# fast"

# ── 0. Check dependencies ───────────────────────────────────────────────────
command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo not found. Install from https://rustup.rs"; exit 1; }
command -v git >/dev/null 2>&1 || { echo "Git not found. Install from https://git-scm.com"; exit 1; }

# ── 1. Build & install the binary ────────────────────────────────────────────
REPO_URL="https://github.com/Hardik-G1/fast.git"
SRC_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd)"
if [ ! -f "$SRC_DIR/Cargo.toml" ]; then
    # Running via pipe (curl | bash) — clone the repo to a temp dir
    echo "Downloading source..."
    TMP_DIR=$(mktemp -d)
    git clone --depth 1 "$REPO_URL" "$TMP_DIR"
    SRC_DIR="$TMP_DIR/fast"
fi
echo "Building fast..."
cargo install --path "$SRC_DIR" --quiet
echo "Binary installed."

# ── 2. Add shell functions to rc files ───────────────────────────────────────
COMMON=$(cat <<'EOF'

# fast-tools
fcd()  { local d; d=$(fast cd); [ -n "$d" ] && cd "$d"; }
fh()   { fast hist; }
ftop() { fast top; }
f()    { local cmd; cmd=$(fast alias run "$1"); if [ -n "$cmd" ]; then eval "$cmd"; else echo "Alias '$1' not found"; fi; }
EOF
)

BASH_HOOK=$(cat <<'EOF'
__fast_hist_record() { [ $? -eq 0 ] && { local cmd; cmd=$(history 1 | sed "s/^ *[0-9]* *//"); fast hist --add "$cmd"; }; }
PROMPT_COMMAND="__fast_hist_record${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
# fast-tools-end
EOF
)

ZSH_HOOK=$(cat <<'EOF'
__fast_hist_record() { fast hist --add "$1"; }
autoload -Uz add-zsh-hook
add-zsh-hook preexec __fast_hist_record
# fast-tools-end
EOF
)

add_to_rc() {
    local rc="$1" hook="$2"
    if [ ! -f "$rc" ]; then return; fi
    local snippet="$COMMON
$hook"
    if grep -q "$MARKER" "$rc" 2>/dev/null; then
        local tmp; tmp=$(mktemp)
        sed "/$MARKER/,/# fast-tools-end/d" "$rc" > "$tmp"
        mv "$tmp" "$rc"
        printf '%s\n' "$snippet" >> "$rc"
        echo "Shell functions updated in $rc"
    else
        printf '%s\n' "$snippet" >> "$rc"
        echo "Shell functions added to $rc"
    fi
}

add_to_rc "$HOME/.bashrc" "$BASH_HOOK"
add_to_rc "$HOME/.zshrc" "$ZSH_HOOK"

# ── 3. Done ───────────────────────────────────────────────────────────────────
echo ""
echo "Done! Reload your shell:"
echo "  source ~/.bashrc   # or ~/.zshrc"
echo ""
echo "Commands available after reload:"
echo "  fcd              - file browser (cd on Enter)"
echo "  fh               - history picker (Enter runs command)"
echo "  ftop             - system monitor"
echo "  f <alias>        - run a saved alias"
echo "  fast alias add <name> <cmd>  - save an alias"
echo "  fast alias list  - list aliases"
