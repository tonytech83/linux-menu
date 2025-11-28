#!/bin/sh -e

setup_dwm_dependecies() {
    echo "Installing DWM-Titus if not already installed"
    case "$PACKAGER" in # Install pre-Requisites
        pacman)
            $ESCALATION_TOOL "$PACKAGER" -S --needed --noconfirm base-devel libx11 libxinerama libxft imlib2 libxcb
            ;;
        apt)
            $ESCALATION_TOOL "$PACKAGER" install -y build-essential libx11-dev libxinerama-dev libxft-dev libimlib2-dev libxcb1-dev libx11-xcb1
            ;;
        dnf)
            $ESCALATION_TOOL "$PACKAGER" groupinstall -y "Development Tools"
            $ESCALATION_TOOL "$PACKAGER" install -y libX11-devel libXinerama-devel libXft-devel imlib2-devel libxcb-devel
            ;;
        *)
            echo "Unsupported package manager: $PACKAGER"
            exit 1
            ;;
    esac
}

setup_dwm_dependecies