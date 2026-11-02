#!/usr/bin/env bash
# 构建 GitCIStatus.app（菜单栏应用包，LSUIElement 不占 Dock）
set -euo pipefail
cd "$(dirname "$0")/.."

swift build -c release

APP="build/GitCIStatus.app"
mkdir -p "$APP/Contents/MacOS"
cp ".build/release/GitCIStatus" "$APP/Contents/MacOS/GitCIStatus"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>zh_CN</string>
    <key>CFBundleExecutable</key><string>GitCIStatus</string>
    <key>CFBundleIdentifier</key><string>com.xinnix.gitcistatus</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>GitCIStatus</string>
    <key>CFBundleDisplayName</key><string>GitCIStatus</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>1.0.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>LSMinimumSystemVersion</key><string>13.0</string>
    <key>LSUIElement</key><true/>
    <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST

codesign --force --sign - "$APP" 2>/dev/null || true
echo "✅ 已生成 $APP"
