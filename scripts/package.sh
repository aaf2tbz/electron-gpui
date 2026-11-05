#!/usr/bin/env bash
# Build a macOS .app bundle (and optionally a .dmg) from this Cargo project.
#
#   scripts/package.sh            # release build -> target/release/<App Name>.app
#   scripts/package.sh --dmg      # ...and target/release/<App Name>-<version>.dmg
#
# Settings come from the environment, with defaults from Cargo.toml:
#   APP_NAME        display name                    (default: "Electron GPUI")
#   APP_BUNDLE_ID   reverse-DNS identifier          (default: dev.example.electron-gpui)
#   APP_VERSION     CFBundleShortVersionString      (default: Cargo.toml version)
#   APP_CATEGORY    LSApplicationCategoryType       (default: public.app-category.developer-tools)
#   APP_ICON        1024x1024 PNG                   (default: assets/AppIcon.png)
#   SIGN_IDENTITY   codesign identity; "-" = ad-hoc (default: -)
#   ENTITLEMENTS    entitlements plist for Developer ID signing (optional)
#   CARGO_PROFILE   release or debug                (default: release)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MAKE_DMG=0
for arg in "$@"; do
  case "$arg" in
    --dmg) MAKE_DMG=1 ;;
    *) echo "Unknown argument: $arg (use --dmg)" >&2; exit 1 ;;
  esac
done

cargo_field() { sed -n "s/^$1 *= *\"\(.*\)\"/\1/p" Cargo.toml | head -1; }
BINARY="$(cargo_field name)"
APP_NAME="${APP_NAME:-Electron GPUI}"
APP_BUNDLE_ID="${APP_BUNDLE_ID:-dev.example.electron-gpui}"
APP_VERSION="${APP_VERSION:-$(cargo_field version)}"
APP_BUILD_VERSION="${APP_VERSION%%-*}"
APP_CATEGORY="${APP_CATEGORY:-public.app-category.developer-tools}"
APP_ICON="${APP_ICON:-$ROOT/assets/AppIcon.png}"
SIGN_IDENTITY="${SIGN_IDENTITY:--}"
CARGO_PROFILE="${CARGO_PROFILE:-release}"

[[ "$APP_NAME" =~ ^[A-Za-z0-9][A-Za-z0-9\ .-]*$ ]] || { echo "Invalid APP_NAME: $APP_NAME" >&2; exit 1; }
[[ "$APP_BUNDLE_ID" =~ ^[A-Za-z0-9.-]+$ ]] || { echo "Invalid APP_BUNDLE_ID: $APP_BUNDLE_ID" >&2; exit 1; }
[[ "$APP_VERSION" =~ ^[0-9]+([.][0-9]+){1,2}(-[A-Za-z0-9.-]+)?$ ]] || { echo "Invalid APP_VERSION: $APP_VERSION" >&2; exit 1; }
command -v cargo >/dev/null || { echo "cargo is required" >&2; exit 1; }

CARGO_ARGS=(--locked)
case "$CARGO_PROFILE" in
  release) CARGO_ARGS+=(--release) ;;
  debug) ;;
  *) echo "CARGO_PROFILE must be release or debug" >&2; exit 1 ;;
esac
# No MACOSX_DEPLOYMENT_TARGET here: with the macOS 27 linker it also applies to
# host proc-macro dylibs and produces ones dyld rejects ("mis-aligned LINKEDIT
# string pool" -> E0463). Info.plist's LSMinimumSystemVersion sets the floor.
cargo build "${CARGO_ARGS[@]}"

OUT="$ROOT/target/$CARGO_PROFILE"
BUNDLE="$OUT/$APP_NAME.app"
CONTENTS="$BUNDLE/Contents"
rm -rf "$BUNDLE"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"
install -m 0755 "$OUT/$BINARY" "$CONTENTS/MacOS/$BINARY"

# PNG -> .icns with the tools that ship with macOS (no Xcode needed).
ICON_PLIST=""
if [ -s "$APP_ICON" ]; then
  ICONSET="$(mktemp -d "${TMPDIR:-/tmp}/appicon.XXXXXX")/AppIcon.iconset"
  trap 'rm -rf "$(dirname "$ICONSET")"' EXIT
  mkdir -p "$ICONSET"
  for px in 16 32 128 256 512; do
    sips -z $px $px "$APP_ICON" --out "$ICONSET/icon_${px}x${px}.png" >/dev/null
    sips -z $((px * 2)) $((px * 2)) "$APP_ICON" --out "$ICONSET/icon_${px}x${px}@2x.png" >/dev/null
  done
  iconutil -c icns "$ICONSET" -o "$CONTENTS/Resources/AppIcon.icns"
  ICON_PLIST="<key>CFBundleIconFile</key><string>AppIcon</string>"
else
  echo "No icon at $APP_ICON; packaging without one" >&2
fi

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>$BINARY</string>
  <key>CFBundleIdentifier</key><string>$APP_BUNDLE_ID</string>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundleDisplayName</key><string>$APP_NAME</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$APP_VERSION</string>
  <key>CFBundleVersion</key><string>$APP_BUILD_VERSION</string>
  $ICON_PLIST
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSApplicationCategoryType</key><string>$APP_CATEGORY</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST
/usr/bin/plutil -lint "$CONTENTS/Info.plist" >/dev/null

# Ad-hoc signing by default, which is enough to run locally. A Developer ID
# identity signs with the hardened runtime and a secure timestamp, as
# notarization requires (then: xcrun notarytool submit ... && xcrun stapler staple).
if [ "$SIGN_IDENTITY" = "-" ]; then
  codesign --force --deep --sign - "$BUNDLE"
else
  SIGN_ARGS=(--force --timestamp --options runtime --sign "$SIGN_IDENTITY")
  [ -n "${ENTITLEMENTS:-}" ] && SIGN_ARGS+=(--entitlements "$ENTITLEMENTS")
  codesign "${SIGN_ARGS[@]}" "$BUNDLE"
fi
codesign --verify --deep --strict "$BUNDLE"
echo "Built $BUNDLE ($APP_BUNDLE_ID $APP_VERSION)"

if [ "$MAKE_DMG" = 1 ]; then
  DMG="$OUT/$APP_NAME-$APP_VERSION.dmg"
  STAGE="$(mktemp -d "${TMPDIR:-/tmp}/dmg.XXXXXX")"
  cp -R "$BUNDLE" "$STAGE/"
  ln -s /Applications "$STAGE/Applications"
  rm -f "$DMG"
  hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
  rm -rf "$STAGE"
  [ "$SIGN_IDENTITY" = "-" ] || codesign --force --timestamp --sign "$SIGN_IDENTITY" "$DMG"
  echo "Built $DMG"
fi
