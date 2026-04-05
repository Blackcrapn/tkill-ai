#!/bin/bash
# TKILL AI Installer for Arch Linux (Hyprland)

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

PROGRAM_NAME="tkill-ai"
REPO_URL="https://github.com/Blackcrapn/tkill-ai.git"
BUILD_DIR="/tmp/tkill-ai-build-$$"
INSTALL_BIN="/usr/local/bin/$PROGRAM_NAME"
CONFIG_DIR="$HOME/.config/tkill-ai"

log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[ OK ]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

check_os() {
    if ! grep -q "ID=arch" /etc/os-release 2>/dev/null; then
        log_error "Only Arch Linux is supported"
    fi
    log_ok "Arch Linux detected"
}

install_system_deps() {
    log_info "Checking system dependencies..."
    local deps=("git" "base-devel" "libxcb" "openssl" "curl" "which" "whisper.cpp")
    local missing=()
    for dep in "${deps[@]}"; do
        if ! pacman -Q "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done
    if [ ${#missing[@]} -gt 0 ]; then
        sudo pacman -S --noconfirm "${missing[@]}" || log_error "Failed to install dependencies"
    fi
    log_ok "System dependencies ready"
}

setup_rust() {
    if ! command -v rustc &>/dev/null; then
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    export PATH="$HOME/.cargo/bin:$PATH"
    log_ok "Rust ready: $(rustc --version)"
}

clone_repo() {
    log_info "Cloning repository..."
    git clone --depth 1 "$REPO_URL" "$BUILD_DIR"
    cd "$BUILD_DIR"
    log_ok "Cloned"
}

build_project() {
    log_info "Building in release mode..."
    cargo build --release --locked
    log_ok "Build successful"
}

install_binary() {
    sudo cp "$BUILD_DIR/target/release/$PROGRAM_NAME" "$INSTALL_BIN"
    sudo chmod 755 "$INSTALL_BIN"
    log_ok "Installed to $INSTALL_BIN"
}

create_config() {
    mkdir -p "$CONFIG_DIR"
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        cat > "$CONFIG_DIR/config.toml" << 'EOF'
# TKILL AI Configuration
# Get your free GitHub token: https://github.com/settings/tokens
[github]
token = "YOUR_TOKEN_HERE"
model = "gpt-4o-mini"

[audio]
input_device = "default"

[behavior]
log_level = "info"
EOF
        chmod 600 "$CONFIG_DIR/config.toml"
        log_ok "Config created at $CONFIG_DIR/config.toml"
        log_warn "Please edit $CONFIG_DIR/config.toml and add your GitHub token"
    else
        log_info "Config already exists"
    fi
}

add_hyprland_hotkey() {
    echo ""
    read -rp "Add Hyprland hotkey (SUPER+SPACE)? [y/N]: " choice </dev/tty || true
    if [[ ! "$choice" =~ ^[Yy]$ ]]; then
        return
    fi
    HYPR_CONFIG="$HOME/.config/hypr/hyprland.conf"
    if [ ! -f "$HYPR_CONFIG" ]; then
        log_warn "Hyprland config not found, skipping"
        return
    fi
    if grep -q "tkill-ai" "$HYPR_CONFIG"; then
        log_warn "TKILL AI already configured in hyprland.conf"
        return
    fi
    cat >> "$HYPR_CONFIG" << EOF

# TKILL AI - Voice Assistant
windowrulev2 = float, class:^(tkill-ai)$
windowrulev2 = size 800 120, class:^(tkill-ai)$
windowrulev2 = center, class:^(tkill-ai)$
windowrulev2 = noborder, class:^(tkill-ai)$
bind = SUPER, SPACE, exec, tkill-ai
EOF
    log_ok "Hotkey added. Run 'hyprctl reload' to apply"
}

cleanup() {
    rm -rf "$BUILD_DIR"
}

main() {
    trap cleanup EXIT
    echo -e "${GREEN}╔══════════════════════════════════╗${NC}"
    echo -e "${GREEN}║        TKILL AI INSTALLER        ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════╝${NC}"
    check_os
    install_system_deps
    setup_rust
    clone_repo
    build_project
    install_binary
    create_config
    add_hyprland_hotkey
    cleanup
    echo ""
    log_ok "Installation finished successfully!"
    echo ""
    echo "  ▶ Run:  tkill-ai"
    echo "  ▶ Config: $CONFIG_DIR/config.toml (add your GitHub token there)"
    echo "  ▶ Hotkey: SUPER+SPACE (if you chose yes)"
    echo ""
}

main "$@"
