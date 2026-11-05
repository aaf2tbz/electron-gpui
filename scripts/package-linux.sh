#!/usr/bin/env bash
# Build a Linux release tarball with the binary, a .desktop entry and an icon.
#
#   scripts/package-linux.sh   # -> target/release/<binary>-<version>-linux-<arch>.tar.gz
#
# Unpack it anywhere and run ./install.sh to install for the current user
# (~/.local), or PREFIX=/usr/local sudo ./install.sh for everyone.
#
# Settings come from the environment, with defaults from Cargo.toml:
#   APP_NAME       display name                  (default: "Electron GPUI")
#   APP_ID         reverse-DNS id; must match APP_ID in src/main.rs
#                                                (default: dev.example.electron-gpui)
#   APP_VERSION    version                       (default: Cargo.toml version)
#   APP_COMMENT    one-line description          (default: Cargo.toml description)
#   APP_CATEGORIES freedesktop categories        (default: Development;)
#   APP_ICON       PNG, ideally 512x512 or more  (default: assets/AppIcon.png)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo_field() { sed -n "s/^$1 *= *\"\(.*\)\"/\1/p" Cargo.toml | head -1; }
BINARY="$(cargo_field name)"
APP_NAME="${APP_NAME:-Electron GPUI}"
APP_ID="${APP_ID:-dev.example.electron-gpui}"
APP_VERSION="${APP_VERSION:-$(cargo_field version)}"
APP_COMMENT="${APP_COMMENT:-$(cargo_field description)}"
APP_CATEGORIES="${APP_CATEGORIES:-Development;}"
APP_ICON="${APP_ICON:-$ROOT/assets/AppIcon.png}"
ARCH="$(uname -m)"

[ "$(uname -s)" = Linux ] || { echo "Run this on Linux (scripts/package.sh packages for macOS)" >&2; exit 1; }
[[ "$APP_ID" =~ ^[A-Za-z0-9.-]+$ ]] || { echo "Invalid APP_ID: $APP_ID" >&2; exit 1; }
grep -q "APP_ID: &str = \"$APP_ID\"" src/main.rs || \
  echo "warning: APP_ID ($APP_ID) differs from src/main.rs; the desktop won't match the window to its icon" >&2

cargo build --locked --release

NAME="$BINARY-$APP_VERSION-linux-$ARCH"
STAGE="$ROOT/target/release/$NAME"
rm -rf "$STAGE"
mkdir -p "$STAGE/bin" "$STAGE/share/applications" "$STAGE/share/icons/hicolor/512x512/apps"
install -m 0755 "target/release/$BINARY" "$STAGE/bin/$BINARY"
strip "$STAGE/bin/$BINARY" 2>/dev/null || true

ICON_OUT="$STAGE/share/icons/hicolor/512x512/apps/$APP_ID.png"
if command -v magick >/dev/null; then
  magick "$APP_ICON" -resize 512x512 "$ICON_OUT"
elif command -v convert >/dev/null; then
  convert "$APP_ICON" -resize 512x512 "$ICON_OUT"
else
  cp "$APP_ICON" "$ICON_OUT" # desktops scale it; install ImageMagick for an exact 512px icon
fi

# Exec is rewritten to the absolute path by install.sh.
cat > "$STAGE/share/applications/$APP_ID.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=$APP_COMMENT
Exec=$BINARY
Icon=$APP_ID
Terminal=false
Categories=$APP_CATEGORIES
StartupWMClass=$APP_ID
DESKTOP

cat > "$STAGE/install.sh" <<INSTALL
#!/bin/sh
# Install $APP_NAME. PREFIX defaults to ~/.local (no root needed).
set -eu
HERE="\$(cd "\$(dirname "\$0")" && pwd)"
PREFIX="\${PREFIX:-\$HOME/.local}"
install -Dm0755 "\$HERE/bin/$BINARY" "\$PREFIX/bin/$BINARY"
install -Dm0644 "\$HERE/share/icons/hicolor/512x512/apps/$APP_ID.png" "\$PREFIX/share/icons/hicolor/512x512/apps/$APP_ID.png"
sed "s|^Exec=.*|Exec=\$PREFIX/bin/$BINARY|" "\$HERE/share/applications/$APP_ID.desktop" > "\$HERE/.desktop.tmp"
install -Dm0644 "\$HERE/.desktop.tmp" "\$PREFIX/share/applications/$APP_ID.desktop"
rm -f "\$HERE/.desktop.tmp"
command -v update-desktop-database >/dev/null && update-desktop-database "\$PREFIX/share/applications" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "\$PREFIX/share/icons/hicolor" 2>/dev/null || true
echo "Installed $APP_NAME to \$PREFIX"
INSTALL
chmod 0755 "$STAGE/install.sh"
cp LICENSE NOTICE "$STAGE/"
cp -R licenses "$STAGE/"

tar -C "$ROOT/target/release" -czf "$ROOT/target/release/$NAME.tar.gz" "$NAME"
echo "Built target/release/$NAME.tar.gz"
