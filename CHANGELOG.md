# Changelog

All notable changes to GitCIStatus are documented here. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

### Added
- **检查更新**：启动时静默查询 GitHub Releases 最新版，有新版在面板顶部出横幅并一键下载对应平台安装包（零签名依赖；完整静默 updater 待 Apple 签名就绪后启用）
- **Launch at login** toggle in Manage → 刷新与通知（std-only 实现：macOS LaunchAgent / Windows 注册表 Run / Linux XDG autostart，下次登录生效）
- System notifications now follow app/system language（中文/English），并修复失败文案的重复计数
- 点击 macOS 通知会激活应用并自动唤起面板（`RunEvent::Reopen`；通知插件桌面端暂无点击回调 API）
- 失败仓库在面板与托盘菜单中置顶（失败 > 错误 > 运行中 > 其余，组内保持配置顺序）
- DMG 打包目标（本地 `npx tauri build` 即可产出 DMG）
- GitHub Actions 发布流水线（tag 触发构建 macOS 通用二进制 → Draft Release；签名/公证步骤已预留）

### Changed
- **「通过 GitHub 登录」成为唯一鉴权方式**：移除手动粘贴 PAT、gh CLI 与环境变量回退，Token 只存系统钥匙串；配置文件遗留 `token` 字段被忽略（下次保存移除）
- 移除轮询循环的逐周期 stderr 调试日志（保留启动路由信息、网络回退事件与前端错误上报）
- 托盘 API 换用非 deprecated 的 `show_menu_on_left_click`

## [0.1.0] - 2026-11-02

初始功能基线（产品化 M0 之前的功能完成态）：

- 托盘五态圆点 + 失败数徽标；面板折叠卡片：状态徽章、检查统计、最新提交
- 失败详情：job ▸ step + 耗时 + 查看日志直链；最近 4 次运行；PR/Issues/Stars
- 系统通知（失败 / 运行中→通过，可分别关闭）
- RFC 8628 Device Flow 浏览器登录（内置 Client ID，Token 入钥匙串）；仓库选择流（/user/repos 勾选）
- 手动批量添加、本地目录扫描；刷新间隔 15s–5min；无 Token 自动降速
- ETag 条件请求；代理主路由 + 直连回退；中英双语 UI
- 产品化 M0：MIT LICENSE、产品方案与路线文档、git 仓库、配置 snake_case 统一（兼容旧 camelCase）、Swift 原生版归档
