#!/bin/zsh
# Build the Glint widget extension (.appex) with swiftc — no Xcode project needed.
# Usage: ./build-appex.sh
set -euo pipefail

SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
SDK="$(xcrun --show-sdk-path)"
OUT="$SRC_DIR/build"

mkdir -p "$OUT"

xcrun -sdk "$SDK" swiftc \
  -swift-version 5 -O \
  -target arm64-apple-macos14.0 \
  -parse-as-library \
  -o "$OUT/glint-widget" \
  "$SRC_DIR/GlintWidget.swift"

APEX="$OUT/GlintWidget.appex"
rm -rf "$APEX"
mkdir -p "$APEX/Contents/MacOS" "$APEX/Contents/Resources"
cp "$OUT/glint-widget" "$APEX/Contents/MacOS/glint-widget"
cp "$SRC_DIR/Info.plist" "$APEX/Contents/Info.plist"

# NOTE: sandbox entitlement only — no app group (widgetContainer transport;
# the unsandboxed host app writes into this extension's container directly).
codesign -f -s GlintLocal --options runtime \
  --entitlements "$SRC_DIR/widget.entitlements" \
  "$APEX"

codesign --verify --strict "$APEX"
codesign -dv --entitlements - "$APEX" 2>&1 | tail -3
echo "== appex built & signed: $APEX"
