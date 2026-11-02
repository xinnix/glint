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
- [ ] 通知点击跳转：Tauri 版通知点击打开对应仓库/运行页（参照 Swift 版 Notifier 行为）
- [ ] 通知文案双语化：Rust 侧通知文案按 `config.lang` 输出，修复「2 项检查未通过 · 2 check(s) failed」重复计数格式
- [ ] 开机自启：集成 tauri-plugin-autostart，管理窗口加开关
- [ ] 托盘图标深浅色适配：macOS template image / 深色模式
- [ ] 稳定性清理：移除生产路径 `eprintln!` 调试日志；限流/网络错误状态在面板可见（紫色态已有，补充文案与恢复提示）
- [ ] 应用图标定稿：自绘产品 Logo（1024px → tauri icon 全套）

### 分发就绪
- [ ] Apple Developer ID 签名 + notarytool 公证（有账号后）
- [ ] DMG 发行物 + GitHub Release 上传
- [ ] Tauri updater 自动更新（公私钥、latest.json 指向 GitHub Releases、面板检查更新入口）
- [ ] GitHub Actions 发布流水线：打 tag → 构建 → 签名 → Draft Release
- [ ] Homebrew Cask 提交

### 门面
- [ ] README 英文化（EN 主体 + 中文链接）、GIF 演示、截图、badges
- [ ] CHANGELOG.md 起始

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
