# GitCIStatus

English | [简体中文](README.zh-CN.md)

**A tray-resident GitHub CI status dashboard** — no more opening the browser to check if CI passed. One glance at the tray icon tells you everything; when something fails, drill down to the failing job ▸ step without ever opening a web page.

> Push, switch away, and relax: it will shout when things go red, and open it to fix.

<!-- TODO(M1): screenshot/GIF placeholder — push → tray turns red → open to see the failing step → jump to logs -->

## Why

Checking CI on GitHub means switching to the browser, finding the repo, clicking Actions, and waiting for it to load — every single time. GitCIStatus compresses that into one glance: tray green, keep working; tray red, fix it first, with the exact failing job ▸ step one click away.

Free and open source (MIT). Runs entirely locally — no server, no telemetry, token stored in the system keychain.

## Features

- **Five-state tray dot** (aggregated worst status across all monitored repos):
  - 🟢 green: all passing · 🔴 red + count: failures (count = failing repos) · 🟠 orange: CI running
  - ⚪ gray: no CI / no data · 🟣 purple hollow: API error (bad token, rate limited, etc.)
- **Click to open the panel** — one collapsible card per repo:
  - CI status badge, check summary (e.g. "8 checks: ✗ 1 ✓ 6 ⊘ 1"), latest commit on the default branch
  - **Failure details**: the exact failing job ▸ step (e.g. `test ▸ Run tests`), duration, direct "View log" link
  - Last 4 workflow runs, PR / Issues / Stars shortcuts
  - Failing repos automatically sorted to the top
- **System notifications**: instant on failure; also on running → pass (each toggleable, localized to your language)
  - Clicking a macOS notification brings up the panel with failure details
- **Zero-config sign-in**: RFC 8628 Device Flow browser authorization (same as `gh auth login --web`), built-in Client ID, then tick the repos you want to monitor
- **Repository management**: fetch your repos into a checklist (forks/archived filtered), bulk paste, optional local directory scan
- **Launch at login**: one toggle in Manage → Refresh & Notifications (applies at next sign-in)
- Bilingual UI (zh/en, follows system language, manually switchable)

## Platforms

| Platform | Status |
|----------|--------|
| macOS | ✅ v1.0 launch platform (current dev builds usable) |
| Linux | 🚧 v1.x (deb / AppImage) |
| Windows | 🚧 v1.x (MSI) |

## Quick start

1. Launch the app → a dot appears in the tray
2. Open **Manage** → **Sign in with GitHub** (browser authorization, no manual token needed)
3. Tick repos from **⟳ Fetch my repos** — changes apply instantly
4. Done. From now on the tray reflects CI status and failures pop a notification

Private repos need a token (sign-in provides one). Public repos work without a token at 60 req/h — the app automatically slows down to one poll per 4 minutes; with a token it's 5000 req/h, and every request is an ETag conditional request, so unchanged responses (304) don't count against the limit.

## Build

The product mainline is the cross-platform version (Tauri 2, Rust + system WebView, no frontend build step):

```bash
cd tauri
npm install       # only installs @tauri-apps/cli
npx tauri dev     # run in debug mode
npx tauri build   # outputs to src-tauri/target/release/bundle/ (.app + DMG)
```

On Linux, install `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev` etc.; on Windows, WebView2 ships with the OS.

The native macOS Swift version is archived (feature-frozen, maintenance only) but still builds:

```bash
swift run             # debug run (notifications unavailable without the .app bundle)
./scripts/build-app.sh
open build/GitCIStatus.app
```

## Configuration

`~/.config/gitcistatus/config.json` (permissions set to 0600 automatically):

```json
{
  "repos": ["owner/repo"],
  "interval_seconds": 30,
  "token": "",
  "notify_on_failure": true,
  "notify_on_success": true,
  "lang": "",
  "oauth_client_id": ""
}
```

Legacy camelCase fields written by the archived Swift version (`intervalSeconds` etc.) are read transparently and normalized to snake_case on the next save. Leave `token` empty — resolution order: `GITHUB_TOKEN`/`GH_TOKEN` env → system keychain → `gh` CLI session → config file plaintext (discouraged).

## Documentation

- [Product design](docs/PRODUCT.md) — positioning, users, competitive landscape, feature map (中文)
- [ROADMAP](docs/ROADMAP.md) — milestones and acceptance criteria (中文)
- [CHANGELOG](CHANGELOG.md)

## Contributing

Issues and PRs welcome. The architecture is intentionally minimal: a Rust backend and a plain HTML/JS/CSS frontend with no build chain. Please run `npx tauri build` before submitting.

## License

[MIT](LICENSE)
