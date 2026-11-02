# GitCIStatus ROADMAP

> 产品决策与背景见 [PRODUCT.md](./PRODUCT.md)。每个里程碑含验收标准；顺序执行，不并行开里程碑。

## M0 工程地基 ✅（2026-11-02）

目标：让仓库具备开源产品的最低门槛，主线收敛为 Tauri。

- [x] `docs/PRODUCT.md` 产品方案（定位/用户/差异化/功能地图/风险）
- [x] `docs/ROADMAP.md` 本文档
- [x] `LICENSE`（MIT）
- [x] 删除废弃脚手架 `tauri-app/`
- [x] Swift 原生版标记归档（`Sources/DEPRECATED.md`，文件不移动、构建不破坏）
- [x] 配置统一：Tauri 版兼容读取 Swift 版 camelCase 字段（`intervalSeconds`/`notifyOnFailure`/`notifyOnSuccess`），保存规范化为 snake_case；保存时补 0600 权限
- [x] `.gitignore` 完善（build 产物、node_modules、target、.build、swiftpm）+ `git init` + 初始提交
- [x] README 更新为产品门面（Tauri 主线格局）

**验收**：`cd tauri && npx tauri build` 通过；含 camelCase 旧字段的 config.json 启动正常且保存后为 snake_case；`git log` 初始提交无构建产物。

## M1 v1.0「macOS 精打磨首发」

目标：macOS 上达到「发给陌生开发者能装、能长期用、能自动更新」的产品水准。

### 体验补齐
- [x] 通知点击唤起面板（2026-11-02）：通知插件桌面端无点击回调（Actions API 仅移动端），落地为 `RunEvent::Reopen` 唤起面板 + 失败仓库置顶（面板与托盘菜单）；直达 URL 跳转留待后续评估原生 delegate 方案
- [x] 通知文案双语化（2026-11-02）：按 `config.lang`（空则跟随系统）输出，修复重复计数文案
- [x] 开机自启（2026-11-02）：不引插件，std-only 实现（macOS LaunchAgent / Windows 注册表 Run / Linux XDG autostart），管理窗口开关
- [ ] 托盘图标深浅色适配：macOS template image / 深色模式
- [x] 稳定性清理（2026-11-02）：移除轮询逐周期 `eprintln!` 调试日志、换用非 deprecated 托盘 API；限流/网络错误面板文案与恢复提示待补
- [ ] 应用图标定稿：自绘产品 Logo（1024px → tauri icon 全套）

### 分发就绪
- [ ] Apple Developer ID 签名 + notarytool 公证（有账号后）
- [x] DMG 发行物（2026-11-02）：bundle targets 加 dmg，本地构建即产出；Release 上传随流水线 tag 触发生效
- [ ] Tauri updater 自动更新（公私钥、latest.json 指向 GitHub Releases、面板检查更新入口）
- [x] GitHub Actions 发布流水线（2026-11-02）：tag → 构建 macOS 通用二进制 → Draft Release，签名/公证步骤已注释预留
- [ ] Homebrew Cask 提交

### 门面
- [x] README 英文化（2026-11-02）：EN 主体 + [README.zh-CN.md]；GIF 演示、截图、badges 待补
- [x] CHANGELOG.md 起始（2026-11-02）

**验收**：陌生开发者在干净 macOS 上五分钟内完成 安装 → 登录 → 勾仓库 → 收到通知 全流程；release 后应用能收到更新提示并升级成功。

## M2 v1.x「跨平台 + 深度」

- [ ] Linux 打包（deb/AppImage）+ Windows 打包（MSI/NSIS），CI 矩阵构建
- [ ] 组织仓库：`/user/orgs` → org 仓库列表并入仓库选择流
- [ ] **PR 视角**：面板按 PR 聚合 CI 状态（哪个 PR 红了），OSS 维护者场景
- [ ] GitHub Enterprise：自定义 API base URL 配置
- [ ] 多账号 Token 管理
- [ ] ETag 缓存持久化（磁盘，重启首轮不耗限额）
- [ ] 产品显示名定夺（GreenDot / CIGuard / 保持 GitCIStatus）

**验收**：至少一名非作者用户在 Linux/Windows 上完成可用性验证；org 仓库与 PR 视角在真实组织仓库上验证。

## M3 v2.0「生态」

- [ ] GitHub App 授权路径：Actions:read + Metadata:read 最小权限、安装 Token 15k/h 限额；OAuth Device Flow 保留为轻量选项
- [ ] 推送模式探索：Webhook 本地接收 或 混合轮询策略（省电省限额）
- [ ] 多 Git 托管适配层：trait 抽象 + GitLab CI 适配（社区可扩展）
- [ ] 团队聚合视图：多仓库失败总览
- [ ] 移动伴侣端调研（iOS/Android 推送）

**验收**：GitHub App 授权路径在个人 + 组织仓库全流程可用；适配层有至少一个第三方托管平台的可用适配器。
