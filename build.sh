#!/bin/zsh
# Builds DeskPet.app and installs it to ~/Applications.
set -e
cd "$(dirname "$0")"

APP="build/DeskPet.app"
rm -rf build
mkdir -p "$APP/Contents/MacOS"

VERSION="1.1.0"

# Universal binary: Apple silicon + Intel.
swiftc -O -target arm64-apple-macos13 main.swift -o build/DeskPet-arm64
swiftc -O -target x86_64-apple-macos13 main.swift -o build/DeskPet-x86_64
lipo -create build/DeskPet-arm64 build/DeskPet-x86_64 -output "$APP/Contents/MacOS/DeskPet"
rm build/DeskPet-arm64 build/DeskPet-x86_64

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>DeskPet</string>
  <key>CFBundleIdentifier</key><string>local.deskpet</string>
  <key>CFBundleExecutable</key><string>DeskPet</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSUIElement</key><true/>
</dict>
</plist>
PLIST

codesign --force --sign - "$APP"

# Release disk image: the app plus an Applications shortcut to drag it onto.
if [[ "$1" == "--dmg" ]]; then
  STAGE="build/dmg"
  mkdir -p "$STAGE"
  cp -R "$APP" "$STAGE/"
  ln -s /Applications "$STAGE/Applications"
  DMG="build/DeskPet-$VERSION.dmg"
  hdiutil create -volname "DeskPet" -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "$DMG" >/dev/null
  rm -rf "$STAGE"
  (cd build && shasum -a 256 "DeskPet-$VERSION.dmg" > "DeskPet-$VERSION.dmg.sha256")
  echo "Built $DMG"
  exit 0
fi

mkdir -p ~/Applications
pkill -x DeskPet 2>/dev/null || true
rm -rf ~/Applications/DeskPet.app
cp -R "$APP" ~/Applications/
echo "Installed to ~/Applications/DeskPet.app"
