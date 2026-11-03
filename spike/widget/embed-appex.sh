#!/bin/zsh
# Embed the built .appex into a Glint.app bundle and re-sign the outer app.
# Usage: ./embed-appex.sh /path/to/Glint.app
set -euo pipefail

APP_PATH="${1:?usage: embed-appex.sh /path/to/Glint.app}"
SRC_DIR="$(cd "$(dirname "$0")" && pwd)"
APEX="$SRC_DIR/build/GlintWidget.appex"

[[ -d "$APEX" ]] || { echo "appex missing — run build-appex.sh first" >&2; exit 1; }
[[ -d "$APP_PATH" ]] || { echo "app not found: $APP_PATH" >&2; exit 1; }

mkdir -p "$APP_PATH/Contents/PlugIns"
rm -rf "$APP_PATH/Contents/PlugIns/GlintWidget.appex"
cp -R "$APEX" "$APP_PATH/Contents/PlugIns/GlintWidget.appex"

# Re-sign outer app (nested appex is already signed; no --deep).
# --options runtime matches what the Tauri bundler produced for GlintLocal.
codesign -f -s GlintLocal --options runtime "$APP_PATH"
codesign --verify --strict "$APP_PATH"
echo "== appex embedded into $APP_PATH"
