#!/usr/bin/env bash

set -e

# Warna untuk output
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}=================================${NC}"
echo -e "${CYAN}      Uninstalasi Clario CLI     ${NC}"
echo -e "${CYAN}=================================${NC}"

# 1. Hentikan dan Hapus Daemon/Service Background (jika ada)
OS="$(uname -s)"
echo -e "Mendeteksi OS: ${YELLOW}${OS}${NC}"

if [ "$OS" = "Darwin" ]; then
    PLIST_PATH="$HOME/Library/LaunchAgents/com.clario.app.plist"
    if [ -f "$PLIST_PATH" ]; then
        echo -e "Menghapus service LaunchAgents macOS..."
        launchctl unload "$PLIST_PATH" 2>/dev/null || true
        rm -f "$PLIST_PATH"
        echo -e "${GREEN}Service berhasil dihapus.${NC}"
    else
        echo -e "Service autostart tidak ditemukan, diloncati."
    fi
    # dirs::config_dir() pada macOS biasanya ke ~/Library/Application Support
    CONFIG_DIR="$HOME/Library/Application Support/clario"
elif [ "$OS" = "Linux" ]; then
    SERVICE_PATH="$HOME/.config/systemd/user/clario.service"
    if [ -f "$SERVICE_PATH" ]; then
        echo -e "Menghapus service Systemd Linux..."
        systemctl --user stop clario.service 2>/dev/null || true
        systemctl --user disable clario.service 2>/dev/null || true
        rm -f "$SERVICE_PATH"
        systemctl --user daemon-reload 2>/dev/null || true
        echo -e "${GREEN}Service berhasil dihapus.${NC}"
    else
        echo -e "Service autostart tidak ditemukan, diloncati."
    fi
    CONFIG_DIR="$HOME/.config/clario"
else
    echo -e "${RED}OS tidak didukung sepenuhnya untuk uninstalasi servis, diloncati.${NC}"
    CONFIG_DIR="$HOME/.config/clario"
fi

# 2. Hapus Binary
BIN_PATH="$HOME/.local/bin/clario"
if [ -f "$BIN_PATH" ]; then
    echo -e "Menghapus binary aplikasi..."
    rm -f "$BIN_PATH"
    echo -e "${GREEN}Binary clario berhasil dihapus.${NC}"
else
    echo -e "Binary clario tidak ditemukan di ~/.local/bin."
fi

# 3. Hapus Konfigurasi
if [ -d "$CONFIG_DIR" ]; then
    echo -e "Menghapus direktori konfigurasi pengguna..."
    rm -rf "$CONFIG_DIR"
    echo -e "${GREEN}Konfigurasi berhasil dihapus.${NC}"
fi

# 4. Hapus data default Clario (jika ada)
DATA_DIR="$HOME/.local/share/clario"
if [ -d "$DATA_DIR" ]; then
    echo -e "Menghapus direktori data (~/.local/share/clario)..."
    rm -rf "$DATA_DIR"
    echo -e "${GREEN}Data berhasil dihapus.${NC}"
fi

ARCHIVE_DIR="$HOME/Clario_Archives"
if [ -d "$ARCHIVE_DIR" ]; then
    echo -e "Menghapus direktori archive default (${ARCHIVE_DIR})..."
    rm -rf "$ARCHIVE_DIR"
    echo -e "${GREEN}Archive default berhasil dihapus.${NC}"
fi

# Fallback Linux config path
if [ -d "$HOME/.config/clario" ]; then
    rm -rf "$HOME/.config/clario"
fi

echo -e "\n${CYAN}================================================================${NC}"
echo -e "${GREEN}Clario CLI telah berhasil dihapus / di-uninstall bersih!${NC}"
echo -e "${CYAN}================================================================${NC}\n"
