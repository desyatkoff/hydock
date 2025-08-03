#!/usr/bin/env bash


set -euo pipefail

IFS=$'\n\t'

if ! command -v sudo &>/dev/null; then
    if command -v doas &>/dev/null; then
        alias sudo="doas"
    else
        exit 1
    fi
fi

clear

echo "#########################################"
echo "#                                       #"
echo "#   _   _           _            _      #"
echo "#  | | | |_   _  __| | ___   ___| | __  #"
echo "#  | |_| | | | |/ _\` |/ _ \ / __| |/ /  #"
echo "#  |  _  | |_| | (_| | (_) | (__|   <   #"
echo "#  |_| |_|\__, |\__,_|\___/ \___|_|\_\  #"
echo "#         |___/                         #"
echo "#                                       #"
echo "#########################################"
echo ""
echo "Welcome to Hydock installer script"

read -rp "Continue? [Y/n] " confirm

[[ -z "$confirm" || "$confirm" =~ ^[Yy]$ ]] || exit 1

echo "Checking if Rust is installed..."

if ! command -v rustup &> /dev/null; then
    echo "Rust is not installed"
    echo "Installing Rust..."

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    export PATH="$PATH:$HOME/.cargo/bin"

    source "$HOME/.cargo/env"

    echo "Rust has been installed"
else
    echo "Rust is already installed"
fi

echo "Checking if script is running in the project directory..."

if [[ ! -d ".git" && ! -f "install.sh" ]]; then
    if [[ ! -d "hydock" ]]; then
        echo "Cloning Hydock repository..."

        if ! command -v git &>/dev/null; then
            echo "Could not find Git. Installing \`git\` package..."

            if command -v pacman &>/dev/null; then
                sudo pacman -S --noconfirm git
            elif command -v apt &>/dev/null; then
                sudo apt update
                sudo apt install -y git
            elif command -v dnf &>/dev/null; then
                sudo dnf install -y git
            elif command -v zypper &>/dev/null; then
                sudo zypper install -y git
            elif command -v xbps-install &>/dev/null; then
                sudo xbps-install -Sy git
            elif command -v eopkg &>/dev/null; then
                sudo eopkg install -y git
            elif command -v apk &>/dev/null; then
                sudo apk add git
            elif command -v nix-env &>/dev/null; then
                nix-env -iA nixpkgs.git
            else
                echo "Could not detect your package manager"
                echo "Please install \`git\` manually"

                exit 1
            fi
        fi

        git clone https://github.com/desyatkoff/hydock.git
    fi

    cd hydock/
fi

echo "Done"

echo "Cleaning old project files..."

cargo clean --verbose || true

[ -f "/usr/bin/hydock" ] && sudo rm -vf /usr/bin/hydock || true

echo "Done"

echo "Compiling Hydock..."

cargo build --release --verbose

echo "Done"

echo "Copying binary file to \`/usr/bin/\`..."

sudo cp -v \
    ./target/release/hydock \
    /usr/bin/hydock

if [[ ! -d "$HOME/.config/hydock/" ]]; then
    mkdir -v ~/.config/hydock/

    echo "Copying default \`config.toml\`..."

    cp -v \
        ./assets/config.toml \
        ~/.config/hydock/config.toml

    echo "Copying default \`style.css\`..."

    cp -v \
        ./assets/style.css \
        ~/.config/hydock/style.css
fi

echo "Done"

echo "Successfully installed Hydock"
echo "Enjoy your new *blazingly fast* dock for Hyprland"
