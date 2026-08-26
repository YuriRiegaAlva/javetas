#!/bin/sh
# Install the latest javetas release binary for Linux or macOS.
set -eu

REPO="YuriRiegaAlva/javetas"

os=$(uname -s)
arch=$(uname -m)

case "$os-$arch" in
    Linux-x86_64)    target="javetas-linux-x86_64" ;;
    Darwin-x86_64)   target="javetas-macos-x86_64" ;;
    Darwin-arm64)    target="javetas-macos-arm64" ;;
    *)
        echo "javetas: unsupported platform: $os $arch" >&2
        exit 1
        ;;
esac

url="https://github.com/$REPO/releases/latest/download/$target.tar.gz"
dest="${XDG_BIN_HOME:-$HOME/.local}/bin"
tmp="$(mktemp -d)"

echo "Downloading javetas for $os $arch ..."
curl -fsSL "$url" -o "$tmp/javetas.tar.gz" || {
    echo "javetas: download failed. Is there a release on github.com/$REPO/releases?" >&2
    rm -rf "$tmp"
    exit 1
}

mkdir -p "$dest"
tar -xzf "$tmp/javetas.tar.gz" -C "$dest" javetas
chmod +x "$dest/javetas"
rm -rf "$tmp"

case ":$PATH:" in
    *":$dest:"*) ;;
    *)
        echo
        echo "Note: $dest is not in your PATH yet."
        echo "Add this line to your shell config (~/.bashrc, ~/.zshrc, ...):"
        echo "  export PATH=\"\$PATH:$dest\""
        ;;
esac

echo
echo "javetas installed to $dest/javetas"
"$dest/javetas" version
