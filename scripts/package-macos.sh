#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:-0.1.0}"
BUILD="${GITHUB_RUN_NUMBER:-local}"
OUT="$ROOT/dist"
APP="$OUT/OpenCraft.app"
DMG="$OUT/OpenCraft-${VERSION}-macos-${BUILD}.dmg"

echo "Building release client..."
cargo build --release -p client

echo "Assembling OpenCraft.app..."
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/assets"
cp target/release/client "$APP/Contents/MacOS/OpenCraft"
cp -R assets/. "$APP/Contents/Resources/assets/"
chmod +x "$APP/Contents/MacOS/OpenCraft"

cat >"$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>OpenCraft</string>
  <key>CFBundleIdentifier</key>
  <string>com.opencraft.client</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>OpenCraft</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

mkdir -p "$OUT"
rm -f "$DMG"
echo "Creating DMG..."
hdiutil create -volname "OpenCraft" -srcfolder "$APP" -ov -format UDZO "$DMG"
echo "Created $DMG"
