#!/bin/bash

echo "Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed. Exiting."
    exit 1
fi

if [ ! -f target/release/quak ]; then
    echo "Error: target/release/quak not found. Exiting."
    exit 1
fi

TEMPLATE_TARGET_PATH="/home/$USER/.local/share/LICENSER/"
mkdir -p "$TEMPLATE_TARGET_PATH"
mv Templates/ "$TEMPLATE_TARGET_PATH"

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Detected macOS. Copying quak to /usr/local/bin..."
    sudo cp target/release/quak /usr/local/bin/
    INSTALL_PATH="/usr/local/bin/quak"
else
    echo "Copying quak to /usr/bin..."
    sudo cp target/release/quak /usr/bin/
    INSTALL_PATH="/usr/bin/quak"
fi

if [ $? -ne 0 ]; then
    echo "Failed to copy quak to $INSTALL_PATH. Exiting."
    exit 1
fi

echo "quakeditor has been installed.🔥"