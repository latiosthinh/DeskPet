#!/bin/zsh
# Builds DeskPet.app and installs it to ~/Applications.
#
#   mac/build.sh          # build and install
#   mac/build.sh --dmg    # build the release disk image instead (build/DeskPet-<version>.dmg)
#
# Needs Xcode's command line tools and Rust (rustup). The pets are WebAssembly modules run by the
# shared Rust host (mac/core), the same files the Windows app runs; Swift (mac/main.swift) is the Mac
# around it.
set -e
cd "$(dirname "$0")/.."   # run from the repo root; output goes to build/

APP="build/DeskPet.app"
VERSION="${DESKPET_VERSION:-1.2.0}"   # the release workflow passes the tag's version
rm -rf "$APP" build/dmg
mkdir -p "$APP/Contents/MacOS"

# The built-in pets (one per category) are modules too; the app embeds them from build/pets.
mac/build-pets.sh mochi ninja moona

# The shared host as a static library, for Apple silicon and Intel.
export MACOSX_DEPLOYMENT_TARGET=13.0
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  rustup target list --installed | grep -qx $t || rustup target add $t
  cargo build --release -p deskpet-mac --target $t
done

# Universal binary: Apple silicon + Intel.
for arch in arm64:aarch64 x86_64:x86_64; do
  swiftc -O -target "${arch%%:*}-apple-macos13" mac/main.swift \
    -import-objc-header mac/core/deskpet.h \
    -L "target/${arch##*:}-apple-darwin/release" -ldeskpet_mac \
    -o "build/DeskPet-${arch%%:*}"
done
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
  <!-- Pet files (<id>.deskpet, from the DeskPet website) open with DeskPet, which adds the pet. -->
  <key>UTExportedTypeDeclarations</key>
  <array>
    <dict>
      <key>UTTypeIdentifier</key><string>local.deskpet.pet</string>
      <key>UTTypeDescription</key><string>DeskPet pet</string>
      <key>UTTypeConformsTo</key><array><string>public.data</string></array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key><array><string>deskpet</string></array>
      </dict>
    </dict>
  </array>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key><string>DeskPet pet</string>
      <key>CFBundleTypeRole</key><string>Viewer</string>
      <key>LSHandlerRank</key><string>Owner</string>
      <key>LSItemContentTypes</key><array><string>local.deskpet.pet</string></array>
    </dict>
  </array>
</dict>
</plist>
PLIST

codesign --force --sign - "$APP"
echo "Built $APP ($(( $(stat -f%z "$APP/Contents/MacOS/DeskPet") / 1024 )) KB)"

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

# Not on a build machine: install it (CI only checks the build).
if [[ -z "$CI" ]]; then
  mkdir -p ~/Applications
  pkill -x DeskPet 2>/dev/null || true
  rm -rf ~/Applications/DeskPet.app
  cp -R "$APP" ~/Applications/
  echo "Installed to ~/Applications/DeskPet.app"
fi
