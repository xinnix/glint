# GitCIStatus 产品方案

> 版本：2026-11-02 定稿 · 决策人：xinnix
> 配套文档：[ROADMAP.md](./ROADMAP.md)（里程碑与验收标准）

## 1. 问题与机会

**原始痛点**：在 GitHub 上跑 CI，每次都要登录网页才能确认过没过——切浏览器、找仓库、点 Actions、等加载，一天打断 N 次心流。

**共性问题**：所有把 CI 跑在 GitHub Actions 上的开发者都有这个痛点，只是频率不同：

| 人群 | 场景 | 频率 |
|------|------|------|
| 个人开发者 | push 完切走干别的，不知道 CI 什么时候红 | 每天 N 次 |
| OSS 维护者 | PR 触发的 CI 散落在几十个仓库里，不知道该先修哪个 | 每天无数次 |
| 团队成员 | 等依赖仓库的 CI 绿了才能合并/部署，反复刷新网页 | 发布日集中爆发 |

**市场验证**：ActionBar（已上架 Mac App Store）、OctoWatch、Gitify（2.6k+ star）、Trailer 等先例说明需求真实存在；但各有明显缺口（见 §4 竞品对比），存在一个「免费开源 + 跨平台 + step 级失败定位」的空位。

## 2. 产品定位

> **驻守系统托盘的 GitHub CI 状态仪表盘：CI 结果看一眼托盘颜色即知，失败时不用开网页就能定位到失败的 job/step。**

**核心承诺（一句话卖点）**：Push 后放心切走干别的——红了会叫你，点开就能修。

**设计三原则**：

1. **一眼可知**：托盘颜色是唯一必读信息，其余信息都在点击之后（渐进披露）。
2. **失败可行动**：红色不只是「红了」，要能直接看到哪个 job 的哪个 step 挂了、跳到日志行。
3. **不打扰**：默认只对失败弹通知；成功通知可选；轮询对 API 限额友好（ETag + 自动降速）。

## 3. 目标用户画像

**主画像：多仓库并行的个人开发者**（作者本人即首个用户）
- 同时推进 2–10 个项目，push 完就切去写别的代码
- 期望：托盘颜色告诉我「现在有没有事」，有事先修，没事继续干活

**次画像：OSS 维护者**
- 仓库多、PR 多，CI 由他人触发
- 期望：知道「哪个仓库红了」，点开看是哪条 workflow 的哪个 job（v1.x 的 PR 视角进一步满足）

**第三画像：团队成员**
- 监控团队主仓/依赖仓的 CI，绿了才敢合并部署
- 期望：组织仓库一站式监控（v1.x org 支持）、绿了通知我

## 4. 竞品对比与差异化

| | ActionBar | OctoWatch | Gitify | Trailer | **GitCIStatus** |
|---|---|---|---|---|---|
| 免费/开源 | 付费·闭源 | 免费·未开源 | 免费·开源 | 免费·开源 | **免费·MIT 开源** |
| 平台 | 仅 macOS | 仅 macOS | 三平台 | 仅 macOS | **macOS 首发 → 三平台**（Tauri） |
| 视角 | workflow 运行 | 单仓库工作流 | 通知收件箱 | PR/CI | **多仓库 CI 聚合** |
| 失败定位 | 到 workflow | 到 workflow | 无（跳网页） | 部分 | **到 job▸step + 日志直链** |
| 上手门槛 | GitHub App 授权 | Token | 通知授权 | — | **Device Flow 零配置登录** |
| 技术栈 | 原生 | SwiftUI | Electron | Swift | **Tauri + 原生前端（轻量）** |
| 维护状态 | 活跃 | 新 | 活跃 | 失修 | 活跃 |

**核心差异化（对用户的一句话）**：

1. **失败时不开网页**——直接看到 `job ▸ step`、耗时，一键跳到日志行。竞品都停在「红了」这一层。
2. **免费开源 + 未来跨平台**——ActionBar 付费且只有 macOS；Gitify 是 Electron 且只做通知收件箱（你得先有通知才能用）。
3. **零配置上手**——内置 Device Flow Client ID，登录 → 勾选仓库 → 即用。
4. **隐私**——纯本地运行、无服务器、无遥测，Token 存系统钥匙串。

## 5. 功能地图

### 5.1 已有核心（保留，已实现）

- 五态托盘圆点（绿/红+失败数/橙/灰/紫）+ 聚合所有监控仓库
- 弹出面板：仓库折叠卡片（状态徽章、检查统计、最新提交）
- **失败详情：job ▸ step + 耗时 + 「查看日志」直链**
- 最近 4 次运行历史、PR/Issues/Stars 快捷入口
- 系统通知（失败 / 运行中→通过，可分别关闭）
- RFC 8628 Device Flow 浏览器登录（内置 Client ID，Token 入钥匙串）
- 仓库选择流：拉取 `/user/repos` 复选勾选（过滤 fork/archived）
- 手动批量粘贴添加、本地目录扫描（可选）
- 刷新间隔 15s–5min 可调；无 Token 自动降速至 240s
- ETag 条件请求（304 不耗限额）；代理主路由 + 直连回退
- 中英双语 UI（跟随系统，可手动切换）

### 5.2 v1.0 缺口（macOS 精打磨，产品化必备）

| 项 | 说明 |
|----|------|
| 通知点击跳转 | Tauri 版通知目前不可点击，补齐点击直达对应页面 |
| 通知文案双语化 | 通知走 Rust 侧，目前中英混杂（如「2 项检查未通过 · 2 check(s) failed」），按 config.lang 输出 |
| 开机自启 | tauri-plugin-autostart + 管理窗口开关 |
| 签名 + 公证 + DMG | Apple Developer ID 签名、notarytool 公证、DMG 发行物（需开发者账号） |
| 自动更新 | Tauri updater，更新源指向 GitHub Releases（latest.json） |
| 正式应用图标 | 产品 Logo（不使用 GitHub 官方 logo，规避商标问题） |
| 托盘图标深浅色适配 | macOS template image / 深色模式适配 |
| 发布流水线 | GitHub Actions：tag → 三步构建 → 签名 → Draft Release |
| README 英文化 | EN 主体 + 中文版链接、截图/动图、badges |
| 稳定性 | 清理生产路径 eprintln 调试日志、限流/网络错误在面板可见 |
| Homebrew Cask | `brew install --cask gitcistatus` |

### 5.3 v1.x（跨平台与深度）

- Linux（deb/AppImage）+ Windows（MSI/NSIS）打包与发布物
- 组织仓库支持：`/user/orgs` 列出 org 仓库一并勾选（团队成员画像）
- **PR 视角**：按 PR 聚合 CI 状态（哪个 PR 的 CI 挂了），OSS 维护者刚需
- GitHub Enterprise 支持（自定义 API base URL）
- 多账号 / Token 管理（个人 + work 账号切换）
- ETag 缓存持久化（重启后第一轮不耗限额）
- 命名候选定夺（见 §8 品牌）

### 5.4 v2.0+（远期）

- GitHub App 授权：只申请 Actions:read + Metadata:read 最小权限，安装类 Token 限额 15k/h（对标 ActionBar 的安全卖点；保留 OAuth Device Flow 作为轻量选项）
- Webhook / 推送模式：本地小服务或轮询+推送混合，进一步省电省限额
- 多 Git 托管适配层：GitLab CI / Forgejo（接口抽象，社区可扩展）
- 团队聚合视图：多仓库失败总览页
- 移动伴侣端（iOS/Android 推送）

## 6. 增长策略

**冷启动（v1.0 发布月）**：
1. GitHub 仓库即门面：EN README、GIF 演示（push → 变红 → 点开看 step → 跳日志，15 秒内讲完价值）、截图、badges
2. Homebrew Cask 降低安装门槛
3. Show HN（Thursday UTC 上午）+ V2EX「分享创造」+ r/golang/r/rust 之外的 devtools 社区 + Product Hunt
4. 话题锚点：以「我再也不用开网页看 CI 了」为叙事核心（真实痛点故事）

**持续**：
- CHANGELOG + GitHub Releases 规范发版，用户可订阅
- good first issue 标签欢迎社区 PR（架构极简：前端无构建链、Rust 单文件模块化，利于贡献）
- 用户配置即口碑：零配置登录体验是传播点（「登录后勾一下就能用」）

## 7. 商业模式与边界

- **MIT 开源，永久免费**（已定稿）：不设付费墙，无遥测，无账号体系
- 未来若探索可持续性：优先「赞助 + 托管服务」形态（如团队版聚合看板），**不给现有功能加锁**
- 分发：GitHub Releases（主）+ Homebrew（辅）；不上架 App Store（签名公证走 Developer ID，保留 updater 自主性）

## 8. 品牌

- 仓库名/命令行名：**GitCIStatus**（保留，SEO 与现有配置路径零成本）
- 产品显示名候选（v1.x 定夺，不阻塞 v1.0）：
  - **GreenDot**（状态栏绿点即品牌记忆点，短、好念）
  - **CIGuard**（CI 哨兵）
  - **Pushly**（push 完就走）
- 商标安全：图标为自绘几何图形（圆点+分支线条），不使用 GitHub logo/章鱼猫；文案用「for GitHub」

## 9. 风险与对策

| 风险 | 对策 |
|------|------|
| GitHub API 限流（60/h 未登录） | 默认 30–60s 轮询 + ETag 304 + 未登录自动降速 240s + 面板内引导 GitHub 登录（唯一鉴权方式，Token 只存钥匙串） |
| GitHub 商标/品牌政策 | 自绘图标、不用官方 logo、命名「for GitHub」式 |
| Apple 签名门槛 | 无开发者账号时发布 ad-hoc 签名 + 「右键打开」引导文档；有账号后补公证 |
| 通知疲劳 | 默认仅失败通知；成功通知默认开但可关；后续可加安静时段 |
| 单人维护不可持续 | MIT + 极简架构吸引贡献者；文档完备（本方案 + ROADMAP） |
| 竞品跟进（ActionBar 开源化等） | 护城河在 step 级失败定位深度 + 跨平台 + 社区，不在单点功能 |

## 10. 里程碑总览

详见 [ROADMAP.md](./ROADMAP.md)。摘要：

- **M0 工程地基**（本次）：文档、LICENSE、git、删废弃代码、配置统一 ✅
- **M1 v1.0「macOS 精打磨首发」**：§5.2 清单 + v1.0.0 Release
- **M2 v1.x「跨平台 + 深度」**：§5.3 清单
- **M3 v2.0「生态」**：§5.4 清单
