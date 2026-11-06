# Changelog

All notable changes to Glint are documented here. Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

（下一批：签名/公证、静默更新、Homebrew Cask、组织仓库、PR 视角）

## [1.3.1-beta.1] - 2026-11-06

### Fixed
- **展开仓库卡片时 Issues 区块渲染两遍**：v1.3.0-beta.1 发布工程拆分提交时，前端 Issues 区块被误加了两份（UI 重构提交误带一份、功能提交又插入一份）——每张卡片展开后出现两个相同的「Issues · n 个开放」列表。删除重复区块，功能行为不变

## [1.3.0-beta.1] - 2026-11-05

### Added
- **仓库卡片内嵌开放 Issues 列表**：展开仓库卡片即可直接查看最近活跃的前 8 个开放 issue（bug / 反馈 / 建议），含真实颜色 label 徽章、评论数、活跃时间；点击跳转对应 issue，区块头「查看全部」打开仓库 issues 页。数据复用现有 issues API 响应（此前仅统计数量），零额外请求、零额外限流消耗；无开放 issue 时显示「暂无开放 issue」确认态
- **面板视觉刷新**：Lucide 风格 SVG 图标体系（页脚 / 链接芯片 / 状态徽章 / 空态全部图标化，currentColor 深浅色自适应）；header 与登录引导嵌入品牌 logo mark（圆角色块 + git-branch 镂空 + 状态彩点）；设计令牌重构（表面分层、color-mix 徽章底色、iOS 风格开关、分组卡片、状态/时间列对齐）；中英文案去除装饰符号

### Fixed
- **dev 模式偶发崩溃**：裸二进制（`tauri dev` / `cargo run`）直启时 UNUserNotificationCenter 可能抛 NSException（bundleProxyForCurrentProcess is nil）直接崩掉整个进程，且是否触发取决于启动时序——bundle 外跳过通知初始化与发送，打包版不受影响

## [1.2.4-beta.1] - 2026-11-05

### Fixed
- **托盘图标重做：实心色块 + 镂空 git-branch，无描边**（[#1](https://github.com/xinnix/glint/issues/1)：原恒定白色描边在浅色菜单栏下难以辨认）
  - macOS 26+（Liquid Glass 玻璃菜单栏，不随深浅色翻转、系统图标恒白）：色块恒定白色，与系统图标风格一致
  - 旧版 macOS：色块随菜单栏深浅自动切换（深色栏白色、浅色栏深灰），系统切换外观时即时重绘
  - 状态彩点保留：绿/红/橙/紫/灰五态一眼可辨；彩点以透明间隙环与色块分离

## [1.2.3-beta.1] - 2026-11-04

### Fixed
- **DMG 窗口几何修正**（背景指引版的实测回调）：
  - 图标压在三步卡上：dmgbuild 实测 `icon_locations` 的 y 是**左上原点**（README 所写「左下原点」不符），直接用与背景图一致的左上坐标
  - 窗口开成 810×760、背景贴左上：`window_rect` 第二元组实为**宽高**（dmgbuild core 直接写 Finder WindowBounds `{{x,y},{w,h}}`），并非文档所述 `(right, top)`
  - 底部说明文字被状态栏遮住：macOS Finder 对 DMG 卷强制加标签栏+状态栏（DS_Store 的 Show* 开关无效）——窗口加高至 660×712，正文只占顶部 620，底部 ~92px 纯背景色安全区

### Changed
- DMG 窗口 660×712；Glint / Applications 图标精确对齐拖拽箭头两端（y=180）

## [1.2.2-beta.1] - 2026-11-04

### Changed
- **DMG 安装窗口背景重做**：复刻官网「安装到 macOS」指引视觉——上半区拖拽箭头（Glint → Applications），下半区三步放行卡（① 点「完成」② 打开系统设置选「隐私与安全性」③ 点「仍要打开」，绿色高亮圈标注点击位置）；窗口 660×560，背景带 Retina @2x 版本（高分屏文字清晰）
- 背景画板源文件 `scripts/dmg-background.html`（浏览器 1320×1120 视口截图 → `dmg-background@2x.png`，`sips` 缩半得 @1x，可复现）

## [1.2.1-beta.1] - 2026-11-04 🔥 替换 1.2.0-beta.1

### Fixed
- **DMG 挂载即弹「已损坏，移到废纸篓」**：macOS 15 起 Gatekeeper 在挂载 quarantined DMG 时就评估包内 App，CI 现造自签证书（untrusted 链）签名的 App 直接被判「已损坏」且无右键 override（v1.1.0-beta.1 起潜伏，1.2.0-beta.1 首次被下载暴露）——发布产物改回无签名（`tauri build --no-sign`），恢复 v1.0.x 行为：挂载干净、首次启动右键 → 打开即可；本地 dev 构建保留 GlintLocal 签名，钥匙串免弹收益不变
- **DMG 安装窗口没有背景图**：bundler 的 DMG 视觉（背景 + 图标坐标）靠 AppleScript 驱动 Finder，CI 无头环境静默失败——背景图其实已复制进卷内 `.background/`，但缺 `.DS_Store` 视图设置。改用 dmgbuild 纯 Python 写视图设置（`dmg-settings.py`，与 conf 的 dmg 块参数一致），无头可靠且已本地目视验收

### Changed
- CI mac 构建只产 App 包（`--bundles app`），DMG 由 dmgbuild 打包（产物名不变 `Glint_<ver>_universal.dmg`，顺带不再有 DMG 签名）

## [1.2.0-beta.1] - 2026-11-04

### Added
- **DMG 安装背景**：安装窗口自定义 660×400 背景图（Glint → Applications 拖拽布局），由 `tauri.conf.json` 的 bundler dmg 配置驱动
- **官网 macOS 首次启动指引**：`site/` 新增中英双语 section——拖拽安装示意 + 未签名应用的放行三步，附右键打开、Apple Silicon「已损坏」`xattr -cr` 两条兜底

### Fixed
- **「检查更新」点击冻住整个 UI**（Tauri v2 同步命令跑在主线程）：`check_update` 的阻塞网络请求（最长 2×20s）移入后台线程；同病相怜的命令一并修复——`device_flow_start`（登录按钮卡顿）、`token_info_cmd`（钥匙串读取）、`get_config`/`save_config`（配置读写）、`autostart_state`/`set_autostart_cmd`（Windows reg 子进程）、`import_local_dir`（目录扫描），统一 `async fn` + `spawn_blocking`；纯内存命令保持同步
- **mac CI 签名挂死**：headless runner 上手搓 `security` + codesign 会因交互授权无人应答而永久挂起（实测 40 分钟超时）——改用 tauri-bundler 原生 `APPLE_CERTIFICATE` 机制：现造带 codeSigning EKU、Developer ID 前缀 CN + OU 的自签证书，bundler 自建临时钥匙串并以显式 `--keychain` 路径签名；Build 步骤加 30 分钟超时兜底

### Changed
- 更新检查的资产名单测试 fixture 对齐 CI 实际产物命名（universal DMG），并补混合 universal + 架构命名资产的用例

## [1.1.0-beta.1] - 2026-11-03 🎉 三端发布（macOS / Windows / Linux）

### Added
- **Windows / Linux 安装包**：bundle targets 扩展为三端全集——Linux 产 deb + AppImage（deb 显式声明 libwebkit2gtk-4.1 / libayatana-appindicator3 运行时依赖）、Windows 产 MSI + NSIS 安装器（currentUser 安装模式免 UAC）；托盘点击后面板定位新增 Windows/Linux 物理坐标实现（任务栏在底部时自动翻转到图标上方；Wayland 受系统限制走默认位置）
- **发布流水线三平台矩阵**（`release.yml`）：tag 触发同时构建 macOS 通用 DMG / Linux deb+AppImage / Windows MSI+NSIS，全部追加到同一 Draft Release；CI 检查矩阵（`ci.yml`）同步扩为三平台编译+测试，Windows/Linux 编译从此进 CI 门禁；macOS CI 现造 GlintLocal 自签证书（产物带稳定 CFBundleIdentifier，优于裸 ad-hoc 随机 id）
- Windows 用户区域读取（注册表 LocaleName）用于通知语言检测；无 LANG 的环境默认英文（原先默认中文）

### Fixed
- **检查更新在三端失配**：资产匹配原先按 `x86_64` 字面查找，而 Tauri 实际产物名是 `Glint_x_y_z_x64_en-US.msi` / `glint_x_y_z_amd64.AppImage`（Linux 包名小写）——Windows/Linux 永远匹配不到。重写为平台无关的纯匹配函数（arch 别名 x86_64↔x64/amd64、aarch64↔arm64，大小写不敏感，Windows 优先 NSIS 安装器，macOS Intel 机回退 universal DMG），并配三端产物名回归单测

### Changed
- Windows 配置目录从 `~/.config/glint` 调整为惯例的 `%APPDATA%\glint`（unix 平台路径不变）
- README 宣告三端可用并列出已知限制（未签名警示、非 macOS 通知点击降级、Wayland 面板定位）

## [1.0.1-beta.1] - 2026-11-03

- 版本号提升至 1.0.1（内容同 1.0.0-beta.1 + 托盘镂空方框新图标），用于验证「检查更新 → 横幅 → 下载安装」动态升级链路

## [1.0.0-beta.1] - 2026-11-03 🎉 首个公开 Beta

### Added
- **官网 landing 页**：`site/` 纯 HTML/CSS/JS 静态页（无构建步骤），GitHub Pages 自动部署（`https://xinnix.github.io/glint/`）；中英双语自动检测+手动切换（模式同应用内 i18n）、跟随系统深浅色、CSS 复刻面板窗口 mockup
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
