# Glint

[English](README.md) | 简体中文 | [官网 Landing 页](https://xinnix.github.io/glint/)

**驻守系统托盘的 GitHub CI 状态仪表盘** —— 不用打开网页，看一眼托盘颜色就知道 CI 过没过；失败时不用开网页就能定位到失败的 job ▸ step。

> Push 后放心切走干别的：红了会叫你，点开就能修。

<!-- TODO(M1): 截图/动图占位——推 GIF：push → 托盘变红 → 点开看失败 step → 一键跳日志 -->

## 为什么

在 GitHub 上跑 CI，每次确认结果都要切浏览器、找仓库、点 Actions、等加载。Glint 把这件事压缩成一次眼动：托盘 🟢 没事继续干活，🔴 有事先修，点开直接看是哪个 job 的哪个 step 挂了。

免费开源（MIT），纯本地运行、无服务器、无遥测，Token 存系统钥匙串。

## 功能

- **托盘五态圆点**（聚合所有监控仓库的最差状态）：
  - 🟢 绿：全部通过 · 🔴 红+数字：有失败（数字=失败仓库数） · 🟠 橙：有运行中 CI
  - ⚪ 灰：仓库无 CI / 无数据 · 🟣 紫空心：API 错误（Token 失效、限流等）
- **点击弹出面板**，每个仓库一张折叠卡片，展开可见：
  - CI 状态徽章、检查统计（如「检查 8 项：✗ 1 ✓ 6 ⊘ 1」）、默认分支最新提交
  - **失败详情**：具体失败的 job ▸ step（如 `test ▸ Run tests`）、耗时、「查看日志」直达
  - 最近 4 次运行、PR / Issues / Stars 快捷入口
  - 失败仓库自动排在最上面，失败的一眼可见
- **系统通知**：失败立刻通知；「运行中 → 通过」也通知（均可关，文案跟随应用语言）
  - 点击通知会唤起面板直达失败详情（macOS）
- **零配置登录**：RFC 8628 Device Flow 浏览器授权（`gh auth login --web` 同款），内置 Client ID，登录后直接勾选要监控的仓库
- **仓库管理**：拉取自己的仓库复选勾选（过滤 fork/archived）
- **开机自启**：管理窗口一键开关（登录时自动启动，下次登录生效）
- 中英双语界面（跟随系统，可手动切换）

## 平台支持

| 平台 | 状态 |
|------|------|
| macOS | ✅ v1.0 首发平台（通用二进制） |
| Linux | ✅ v1.1（deb / AppImage） |
| Windows | ✅ v1.1（MSI / NSIS 安装器） |

## 已知限制

- **尚未代码签名**（Apple Developer ID / Windows Authenticode 待开发者账号）：macOS 首次启动需右键 → 打开；Windows 可能弹 SmartScreen 警示，点「更多信息」→「仍要运行」。
- **点击通知唤起面板**目前仅 macOS（原生 UNUserNotificationCenter delegate）。Windows/Linux 点击通知仅激活应用，可从托盘菜单「面板 Panel」打开面板。
- **Linux**：Wayland 下无法把面板定位到托盘图标下方（无全局坐标），面板出现在系统默认位置；X11 完整支持。GNOME 的 AppIndicator 托盘左键直接弹菜单，用菜单里的「面板 Panel」项。

## 快速上手

1. 启动应用 → 托盘出现圆点
2. 打开「管理」→ 点「通过 GitHub 登录」（浏览器授权，无需手动填 Token）
3. 从「⟳ 拉取我的仓库」列表勾选要监控的仓库，即时生效
4. 完成。之后 CI 状态变化托盘即变，失败会弹通知

私有仓库需要 Token（登录即得）。公共仓库无 Token 也能用，限流 60 次/小时，应用自动放慢到 4 分钟一刷；带 Token 限额 5000 次/小时，且全部请求走 ETag 条件查询，无变化（304）不耗限额。

## 构建

主线为跨平台版（Tauri 2，Rust + 系统 WebView，前端无构建步骤）：

```bash
cd tauri
npm install       # 仅安装 @tauri-apps/cli
npx tauri dev     # 调试运行
npx tauri build   # 产物在 src-tauri/target/release/bundle/（DMG / deb+AppImage / MSI+NSIS，按平台）
```

Linux 需 `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf` 等系统依赖；Windows 需 WebView2（系统自带）。

原生 macOS Swift 版已归档（停止新功能，仅维护），仍可构建：

```bash
swift run             # 调试运行（无 .app 包时通知不可用）
./scripts/build-app.sh
open build/Glint.app
```

## 配置

`~/.config/glint/config.json`（自动设为 0600）：

```json
{
  "repos": ["owner/repo"],
  "interval_seconds": 30,
  "notify_on_failure": true,
  "notify_on_success": true,
  "lang": "",
  "oauth_client_id": ""
}
```

**「通过 GitHub 登录」是唯一鉴权方式**：Token 经 RFC 8628 Device Flow 获取，只存系统钥匙串——无手动粘贴 Token、无环境变量、无配置文件明文。旧配置里的 `token` 字段会被忽略（下次保存时移除）；归档 Swift 版的 camelCase 旧字段（`intervalSeconds` 等）兼容读取，下次保存规范化为 snake_case。

## 文档

- [产品方案](docs/PRODUCT.md) — 定位、目标用户、竞品对比、功能地图
- [ROADMAP](docs/ROADMAP.md) — 里程碑与验收标准
- [CHANGELOG](CHANGELOG.md) — 变更记录

## 贡献

Issue / PR 欢迎提交。架构刻意保持极简：Rust 后端、前端纯 HTML/JS/CSS 无构建链，改动门槛低。提交前请跑 `npx tauri build` 确认可构建。

## License

[MIT](LICENSE)
