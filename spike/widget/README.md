# Widget Spike(2026-11-03)

验证「Tauri(非沙箱)主程序 + 手搓 WidgetKit 扩展 + widgetContainer 数据通路」在本机(macOS 27)是否可行。

## 结论:✅ 全部程序化验证通过

| 检查项 | 结果 |
|---|---|
| swiftc 编译 .appex(无 Xcode 工程) | ✅ CLT SDK 带 WidgetKit,一次通过 |
| GlintLocal 签名 appex(仅 app-sandbox entitlement) | ✅ |
| 注入 `Contents/PlugIns/` + 重签外层 | ✅ `codesign --verify --strict` 通过 |
| pluginkit 注册 | ✅ `io.github.glint.widget(1.0.1)`,`+` 前缀 = 已启用 |
| **chronod `extensionsPendingDescriptorRefetch`** | ✅ **为空**(CodexBar #1173 卡死点,未复现) |
| 容器由系统自动创建 | ✅ `~/Library/Containers/io.github.glint.widget`(chronod 首次接触即建) |
| 宿主直写扩展沙箱容器(widgetContainer 路线) | ✅ 主程序无沙箱可自由写入 `Data/widget_data.json` |
| 日志沙箱/entitlement 报错 | ✅ 无("not entitled to run in the App Sandbox" 未出现) |

## 流程(可复用)

```bash
./build-appex.sh                      # swiftc 编译 + 打 appex + 签名
./embed-appex.sh <path-to-Glint.app>  # 注入 PlugIns + 重签外层
pluginkit -m -p com.apple.widgetkit-extension | grep glint   # 验证注册
pluginkit -e use -i io.github.glint.widget                   # 显式启用(保险)
```

数据写入(宿主侧,模拟生产 poller):

```bash
cp widget_data.json ~/Library/Containers/io.github.glint.widget/Data/
```

widget 读侧用 `NSHomeDirectory()/widget_data.json`——沙箱内 HOME 即容器 Data 目录。

## 生产化时替换的部分

- spike 的 `widget_data.json` → poller 每轮写真实 snapshot(`RepoState` 已 Serialize),schema 扩展为完整状态
- 加 `WidgetCenter.shared.reloadAllTimelines()`(经 objc2-widget-kit 或侧车 Swift helper),**只在状态变化时调**(复用通知 diff 点;每轮调会被日预算节流)
- spike 视图 → 三尺寸正式布局 + 双语
- `arm64` → universal binary(CI 出通用产物)
- 可选:改用 `tauri-plugin-widgets`(s00d)接管构建管道;spike 已证明手搓管道同样可行,零依赖

## 已知边界

- 主程序不运行时 widget 显示陈旧快照(可接受;公开仓库未来可让 widget 无 token 自拉)
- 扩展进程只在画廊预览/时间线需要时才 spawn,注册成功≠立即运行(正常)
