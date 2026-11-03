# Changelog

All notable changes to GitCIStatus are documented here. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

### Added
- **失败直接复制错误，不用跳网页**：失败详情新增「复制错误」，一键拉取失败 job 的日志并提取错误行（优先 `##[error]` 标记行，否则取日志末尾），连同仓库/提交/job▸step 上下文复制到剪贴板；日志拉取失败时回退为复制摘要（仓库、job、链接）
- **登录验证码一键复制**：设备流授权页的大字验证码旁新增复制按钮，免去手动选中誊抄
- **检查更新**：启动时静默查询 GitHub Releases 最新版，有新版在面板顶部出横幅并一键下载对应平台安装包（零签名依赖；完整静默 updater 待 Apple 签名就绪后启用）
- **Launch at login** toggle in Manage → 刷新与通知（std-only 实现：macOS LaunchAgent / Windows 注册表 Run / Linux XDG autostart，下次登录生效）
- System notifications now follow app/system language（中文/English），并修复失败文案的重复计数
- 点击 macOS 通知会激活应用并自动唤起面板（`RunEvent::Reopen`；通知插件桌面端暂无点击回调 API）
- 失败仓库在面板与托盘菜单中置顶（失败 > 错误 > 运行中 > 其余，组内保持配置顺序）
- DMG 打包目标（本地 `npx tauri build` 即可产出 DMG）
- GitHub Actions 发布流水线（tag 触发构建 macOS 通用二进制 → Draft Release；签名/公证步骤已预留）

### Fixed
- **双屏时托盘点击后面板错位**：tray-icon 上报的点击/图标坐标是「全局逻辑坐标 × 托盘屏 scale」，tao 的 `set_position(PhysicalPosition)` 又按窗口当前屏 scale 反推逻辑点，两屏缩放不同时错位；现还原为全局逻辑点并以 `LogicalPosition` 定位，面板夹进托盘所在屏幕可见范围
- **Token 从未真正落盘**：`keyring = "3"` 未启用任何平台后端 feature，实际编译出 keyring 的内存 mock 存储——登录后重启即失效、`security` CLI 查无此条。现已启用 `apple-native` / `windows-native` / `linux-native`，Token 真正存入系统钥匙串（login keychain），并新增跨进程持久化探针回归测试（`cargo test keyring_roundtrip`）
- 设备流登录失败时错误显示在错误视图（管理页）而用户停留在引导页，表现为「无反应」；现已在引导页内红字提示

### Changed
- **项目更名 GitCIStatus → Glint**（2026-11-03）：仓库 `xinnix/glint`，应用名/包名/标识符（`io.github.glint`）/托盘菜单/通知全面更名；启动时自动迁移——配置目录 `~/.config/gitcistatus` → `~/.config/glint`、钥匙串 service 同步迁移、清理旧名 LaunchAgent plist。slogan：CI status at a glance
- **全新应用图标**：产品视觉定稿——深色圆角方形 + Feather 风格 git-branch 白描边 + 绿色状态点（`scripts/render-appicon.swift` 可复现渲染全套尺寸）
- **托盘图标重设计**：镂空圆角方形外框 + 内嵌 Feather 风格 git-branch，全部描边线条、恒定白色 + 半透明深色外沿（深浅菜单栏均可见，不再依赖系统外观探测——半透明菜单栏+深色壁纸场景下探测会误判）
- 403 限流时面板顶部出提示横幅（未登录附「去登录」入口）
- **管理页添加仓库只保留「从 GitHub 选择」**：移除手动输入 owner/repo 与扫描本地 code 目录两个入口；改为折叠面板（默认收起、首次展开自动拉取仓库列表），勾选完 1.5s 后自动收起，不再常驻占用页面
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
