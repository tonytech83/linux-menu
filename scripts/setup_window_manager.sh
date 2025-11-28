#!/bin/sh -e

setup_window_manager() {
    echo "Setting up Xorg"
    case "$PACKAGER" in
        pacman)
            $ESCALATION_TOOL "$PACKAGER" -S --needed --noconfirm xorg-xinit xorg-server
            ;;
        apt)
            $ESCALATION_TOOL "$PACKAGER" install -y xorg xinit
            ;;
        dnf)
            $ESCALATION_TOOL "$PACKAGER" install -y xorg-x11-xinit xorg-x11-server-Xorg
            ;;
        *)
            echo "Unsupported package manager: $PACKAGER"
            exit 1
            ;;
    esac
    echo "Xorg installed successfully"  
}

setup_window_manager