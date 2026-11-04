# CI 无头产 DMG：纯 Python 写 .DS_Store（背景图 + 图标坐标），不依赖 AppleScript/Finder
# （bundler 的 DMG 视觉靠 osascript 驱动 Finder，无头 runner 上静默失败——背景图进
# 了 .background/ 但视图设置缺失，表现为「背景图不显示」）
#
# 视觉参数与 tauri.conf.json 的 bundle.macOS.dmg 保持一致（后者服务本地 bundler 构建）。
# dmgbuild 坐标系以窗口内容**左下角**为原点；conf 的 *Position 是左上角原点：
#   y_bottom = window_height(400) - y_top = 400 - 170 = 230
# 路径由 release.yml 以 -D 注入：-Dapp=<Glint.app 路径> -Dbackground=<背景图路径>

files = [defines["app"]]
background = defines["background"]
window_rect = ((150, 200), (810, 600))
icon_size = 90
icon_locations = {
    "Glint.app": (180, 230),
    "Applications": (480, 230),
}
symlinks = { "Applications": "/Applications" }
format = "UDZO"
