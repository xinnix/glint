#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Glint 跨平台版（Tauri）：托盘 + 面板监控 GitHub CI
// 功能与 macOS 原生版对齐：check-runs 聚合、失败 job/step 定位、ETag 条件请求、
// 状态变化通知、Token 解析（环境变量 → 钥匙串 → gh CLI → 配置文件）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tauri::menu::{IsMenuItem, Menu, MenuItem, MenuEvent, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(target_os = "macos")]
use tauri::RunEvent;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
#[cfg(not(target_os = "macos"))]
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "macos")]
mod macos_notify;
mod tray_icon;

// ============ 前端数据模型（字段名即 JSON key） ============

#[derive(Debug, Clone, Serialize, Default)]
struct RunRow {
    id: i64,
    title: String,
    state: String,
    url: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct FailedCheck {
    id: i64,
    name: String,
    url: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct FailedJob {
    id: i64,
    name: String,
    failed_step: Option<String>,
    url: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct IssueLabel {
    name: String,
    color: String, // GitHub 返回的 6 位 hex（如 "d73a4a"）
}

#[derive(Debug, Clone, Serialize, Default)]
struct IssueBrief {
    number: i64,
    title: String,
    labels: Vec<IssueLabel>,
    url: Option<String>, // html_url
    created_at: Option<String>,
    updated_at: Option<String>,
    comments: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
struct RepoState {
    repo: String,
    status: String, // loading / running / success / failed / no_checks / error
    branch: Option<String>,
    head_sha: Option<String>,
    commit_subject: Option<String>,
    repo_url: Option<String>,
    rows: Vec<RunRow>,
    open_prs: i64, // -1 = 未知（未拉取）
    open_issues: i64,
    issues: Vec<IssueBrief>,
    stars: i64,
    pushed_at: Option<String>,
    failed_count: i64,
    top_fail_url: Option<String>,
    error_text: Option<String>,
    failed_checks: Vec<FailedCheck>,
    failed_jobs: Vec<FailedJob>,
    total_checks: i64,
    passed_checks: i64,
    skipped_checks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    repos: Vec<String>,
    // alias 兼容 Swift 版（已归档）写入的 camelCase 旧字段；保存时统一输出 snake_case
    #[serde(alias = "intervalSeconds")]
    interval_seconds: u64,
    #[serde(alias = "notifyOnFailure")]
    notify_on_failure: bool,
    #[serde(alias = "notifyOnSuccess")]
    notify_on_success: bool,
    lang: String,
    oauth_client_id: String, // Device Flow 用的 OAuth App Client ID（公开值，无需 secret）
}

impl Default for Config {
    fn default() -> Self {
        Self {
        repos: vec![],
        interval_seconds: 30,
        notify_on_failure: true,
            notify_on_success: true,
            lang: String::new(),
            oauth_client_id: String::new(),
        }
    }
}

#[cfg(test)]
mod keyring_probe {
    // 隔离探针：验证 keyring 平台后端真的持久化（曾抓到过 mock 后端的严重 bug）。
    // 用独立 account，不碰真实 token。会触碰真实钥匙串，CI 跳过：
    // 本地显式运行 `cargo test keyring_roundtrip -- --ignored`
    #[test]
    #[ignore]
    fn keyring_roundtrip() {
        let entry = keyring::Entry::new("glint", "token_probe").unwrap();
        entry.set_password("probe-value-12345").unwrap();
        let got = entry.get_password().unwrap();
        assert_eq!(got, "probe-value-12345");
        let _ = entry.delete_credential();
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    // 归档 Swift 版写入的 camelCase 旧配置必须能无损读取
    #[test]
    fn reads_legacy_camel_case_fields() {
        let raw = r#"{"repos":["a/b"],"intervalSeconds":60,"notifyOnFailure":false,"notifyOnSuccess":true}"#;
        let c: Config = serde_json::from_str(raw).unwrap();
        assert_eq!(c.repos, vec!["a/b"]);
        assert_eq!(c.interval_seconds, 60);
        assert!(!c.notify_on_failure);
        assert!(c.notify_on_success);
    }

    #[test]
    fn reads_snake_case_fields() {
        let raw = r#"{"interval_seconds":120,"notify_on_failure":true}"#;
        let c: Config = serde_json::from_str(raw).unwrap();
        assert_eq!(c.interval_seconds, 120);
        assert!(c.notify_on_failure);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let c: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(c.interval_seconds, 30);
        assert!(c.notify_on_failure);
        assert!(c.notify_on_success);
        assert!(c.repos.is_empty());
    }
    // 「复制错误」摘录：错误行向上带 15 行上下文（根因常在错误前执行的输出里）、
    // 向下带 5 行，剥离时间戳与 ##[error] 标记；无标记时回退日志末尾
    #[test]
    fn excerpt_prefers_error_annotation_lines() {
        let log = concat!(
            "2026-11-03T01:02:03.1234567Z ##[group]Run cargo test\n",
            "2026-11-03T01:02:04.0000000Z    Compiling glint v1.0.0\n",
            "2026-11-03T01:02:05.0000000Z ##[error]test failed: expected 2, got 3\n",
            "2026-11-03T01:02:05.1000000Z ##[error]error: build failed\n",
            "2026-11-03T01:02:06.0000000Z ##[endgroup]\n",
        );
        let ex = extract_error_excerpt(log, 30, 4000);
        // 错误行前的上下文（Run cargo test / Compiling）一并保留；
        // 仅含 ##[...] 标记的行剥离后为空，被过滤
        assert_eq!(
            ex,
            "Run cargo test\n   Compiling glint v1.0.0\ntest failed: expected 2, got 3\nerror: build failed"
        );
    }

    #[test]
    fn excerpt_falls_back_to_log_tail() {
        let log = "l1\nl2\nl3\nl4\nl5\n";
        assert_eq!(extract_error_excerpt(log, 2, 4000), "l4\nl5");
        // 无时间戳前缀的行原样保留
        let plain = "plain line A\nplain line B\n";
        assert_eq!(extract_error_excerpt(plain, 30, 4000), "plain line A\nplain line B");
    }

    #[test]
    fn excerpt_respects_char_cap() {
        let log = "2026-11-03T01:02:03.1234567Z ##[error]aaaaaaaaaa\n2026-11-03T01:02:03.1234567Z ##[error]bbbbbbbbbb\n";
        let ex = extract_error_excerpt(log, 30, 12);
        assert!(ex.chars().count() <= 13); // 内容 + 可能的省略号
        assert!(ex.starts_with("aaaaaaaaaa"));
    }
}

#[cfg(test)]
mod issue_briefs_tests {
    use super::*;

    // /issues 端点会混入 PR（带 pull_request 键），必须过滤；计数与列表同源
    #[test]
    fn filters_out_pull_requests_and_counts_real_issues() {
        let raw = r#"[
            {"number":1,"title":"real bug","pull_request":{"url":"x"}},
            {"number":2,"title":"real issue","html_url":"https://github.com/o/r/issues/2"}
        ]"#;
        let v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let (total, briefs) = parse_issue_briefs(&v);
        assert_eq!(total, 1);
        assert_eq!(briefs.len(), 1);
        assert_eq!(briefs[0].number, 2);
        assert_eq!(briefs[0].title, "real issue");
    }

    // 面板展示按最近活跃（updated_at）降序；总数不截断、列表截前 8
    #[test]
    fn sorts_by_updated_at_and_caps_at_8() {
        let mut raw = String::from("[");
        for n in 1..=10 {
            // number=1 的 updated_at 最新（2026-01-10），number=10 最旧（2026-01-01）
            raw.push_str(&format!(
                r#"{{"number":{},"title":"t{}","updated_at":"2026-01-{:02}T00:00:00Z"}},"#,
                11 - n,
                11 - n,
                n
            ));
        }
        raw.pop(); // 去尾逗号
        raw.push(']');
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let (total, briefs) = parse_issue_briefs(&v);
        assert_eq!(total, 10);
        assert_eq!(briefs.len(), 8);
        // 降序后首位是最近活跃的 number=1，第 8 位是 number=8
        assert_eq!(briefs[0].number, 1);
        assert_eq!(briefs[7].number, 8);
    }

    // label 保留 name + color（前端按真实色渲染徽章）
    #[test]
    fn extracts_label_names_and_colors() {
        let raw = r#"[{"number":7,"title":"x","labels":[{"name":"bug","color":"d73a4a"},{"id":9},{"name":"P1","color":"ffabcd"}]}]"#;
        let v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let (_, briefs) = parse_issue_briefs(&v);
        assert_eq!(briefs[0].labels.len(), 3);
        assert_eq!(briefs[0].labels[0].name, "bug");
        assert_eq!(briefs[0].labels[0].color, "d73a4a");
        // 缺 name/color 的 label 回退为空串，不 panic
        assert_eq!(briefs[0].labels[1].name, "");
        assert_eq!(briefs[0].labels[2].color, "ffabcd");
    }

    // 非数组响应 / 空数组：总数 0、列表空
    #[test]
    fn handles_empty_and_non_array_responses() {
        let (total, briefs) = parse_issue_briefs(&serde_json::json!({"message": "moved"}));
        assert_eq!(total, 0);
        assert!(briefs.is_empty());
        let (total, briefs) = parse_issue_briefs(&serde_json::json!([]));
        assert_eq!(total, 0);
        assert!(briefs.is_empty());
    }
}

#[derive(Debug, Clone, Serialize)]
struct TokenInfo {
    present: bool,
    source: String, // keychain（GitHub 登录）/ none
}

#[derive(Debug, Clone, Serialize)]
struct Snapshot {
    repos: Vec<RepoState>,
    last_refresh_ms: u64,
    refreshing: bool,
}

struct Shared {
    snapshot: Mutex<Vec<RepoState>>,
    last_refresh_ms: Mutex<u64>,
    refreshing: AtomicBool,
    wake: Mutex<mpsc::Sender<()>>,
    device_flow_cancel: Mutex<Option<Arc<AtomicBool>>>,
}

// ============ GitHub API DTO（字段名与 API 一致） ============

#[derive(Debug, Clone, Deserialize, Default)]
struct CheckRun {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    head_sha: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    started_at: Option<String>,
    #[serde(default)]
    completed_at: Option<String>,
}

// ============ 配置 ============

fn config_path() -> std::path::PathBuf {
    let mut p = config_dir_base();
    let _ = std::fs::create_dir_all(&p);
    p.push("config.json");
    p
}

/// 配置目录：unix 保持 ~/.config/glint（历史路径，已有用户数据在此，勿改）；
/// Windows 用惯例位置 %APPDATA%\glint（GUI 进程没有 HOME 也没有 .config 惯例）
#[cfg(windows)]
fn config_dir_base() -> std::path::PathBuf {
    dirs::config_dir()
        .map(|d| d.join("glint"))
        .unwrap_or_else(|| std::path::PathBuf::from("glint"))
}

#[cfg(not(windows))]
fn config_dir_base() -> std::path::PathBuf {
    let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push(".config/glint");
    p
}

fn load_config() -> Config {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config_file(config: &Config) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), json).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        // token 不入此文件，但 repos 等信息仍属隐私，与 Swift 版保持一致的 0600 权限
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(config_path(), std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

// ============ Token：唯一来源是「通过 GitHub 登录」（Device Flow），存系统钥匙串 ============

fn keyring_entry() -> Option<keyring::Entry> {
    keyring::Entry::new("glint", "token").ok()
}

/// 一次性迁移（2026-11-03 产品更名 Glint ← GitCIStatus）：
/// 配置目录 ~/.config/gitcistatus → ~/.config/glint；
/// 钥匙串 service gitcistatus → glint（读到旧 token 即复制并删旧条目）；
/// 清理旧名的开机自启 LaunchAgent plist。
fn migrate_from_legacy() {
    if let Some(home) = dirs::home_dir() {
        let old_dir = home.join(".config/gitcistatus");
        let new_dir = home.join(".config/glint");
        if old_dir.is_dir() && !new_dir.exists() {
            let _ = std::fs::create_dir_all(new_dir.parent().unwrap());
            if std::fs::rename(&old_dir, &new_dir).is_ok() {
                eprintln!("[migrate] config dir ~/.config/gitcistatus -> ~/.config/glint");
            }
        }
        let legacy_plist = home.join("Library/LaunchAgents/io.github.gitcistatus.plist");
        if legacy_plist.exists() {
            let _ = std::fs::remove_file(&legacy_plist);
            eprintln!("[migrate] removed legacy LaunchAgent plist");
        }
    }
    if let (Ok(new), Ok(old)) = (
        keyring::Entry::new("glint", "token"),
        keyring::Entry::new("gitcistatus", "token"),
    ) {
        if new.get_password().is_err() {
            if let Ok(token) = old.get_password() {
                if !token.is_empty() && new.set_password(&token).is_ok() {
                    let _ = old.delete_credential();
                    eprintln!("[migrate] keychain token service gitcistatus -> glint");
                }
            }
        }
    }
}

fn resolve_token() -> Option<String> {
    let entry = keyring_entry()?;
    match entry.get_password() {
        Ok(t) if !t.is_empty() => {
            eprintln!("[keychain] get_password ok ({} chars)", t.len());
            Some(t)
        }
        Ok(_) => {
            eprintln!("[keychain] get_password ok but EMPTY");
            None
        }
        Err(err) => {
            // 读失败不能静默：写成功读失败是「登录成功但卡在引导页」的直接嫌疑
            eprintln!("[keychain] get_password FAILED: {err}");
            None
        }
    }
}

fn token_info() -> TokenInfo {
    if resolve_token().is_some() {
        TokenInfo { present: true, source: "keychain".into() }
    } else {
        TokenInfo { present: false, source: "none".into() }
    }
}

// ============ GitHub 客户端（ETag 条件请求缓存） ============

type EtagMap = HashMap<String, (String, serde_json::Value)>;

fn etag_cache() -> &'static Mutex<EtagMap> {
    static CACHE: OnceLock<Mutex<EtagMap>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// macOS：读取系统代理（scutil --proxy）；其他平台读环境变量
#[cfg(target_os = "macos")]
fn system_https_proxy() -> Option<String> {
    let out = std::process::Command::new("scutil").arg("--proxy").output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let get = |key: &str| -> Option<String> {
        text.lines()
            .find(|l| l.trim_start().starts_with(key))
            .and_then(|l| l.split(':').nth(1))
            .map(|v| v.trim().to_string())
    };
    if get("HTTPSEnable").as_deref() != Some("1") {
        return None;
    }
    let host = get("HTTPSProxy")?;
    let port = get("HTTPSPort").unwrap_or_else(|| "80".into());
    if host.is_empty() || host == "0.0.0.0" {
        return None;
    }
    Some(format!("http://{}:{}", host, port))
}

#[cfg(not(target_os = "macos"))]
fn system_https_proxy() -> Option<String> {
    for key in ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"] {
        if let Ok(v) = std::env::var(key) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

fn proxy_url() -> Option<String> {
    static URL: OnceLock<Option<String>> = OnceLock::new();
    URL.get_or_init(system_https_proxy).clone()
}

fn base_agent() -> ureq::AgentBuilder {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
}

/// 主路由：有系统代理则走代理，否则直连
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        if let Some(proxy) = proxy_url() {
            eprintln!("[net] primary route via proxy {}", proxy);
            if let Ok(p) = ureq::Proxy::new(&proxy) {
                return base_agent().proxy(p).build();
            }
        }
        eprintln!("[net] primary route direct");
        base_agent().build()
    })
}

/// 回退路由：主路由走代理时回退直连，反之无回退
fn fallback_agent() -> Option<&'static ureq::Agent> {
    static FB: OnceLock<Option<ureq::Agent>> = OnceLock::new();
    FB.get_or_init(|| {
        if proxy_url().is_some() {
            eprintln!("[net] fallback route direct enabled");
            Some(base_agent().build())
        } else {
            None
        }
    })
    .as_ref()
}

fn get_json_once(
    a: &ureq::Agent,
    url: &str,
    token: Option<&str>,
    etag: Option<&str>,
) -> Result<(serde_json::Value, Option<String>), String> {
    let mut req = a
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", "Glint/1.0");
    if let Some(t) = token {
        req = req.set("Authorization", &format!("Bearer {}", t));
    }
    if let Some(e) = etag {
        req = req.set("If-None-Match", e);
    }
    match req.call() {
        Ok(resp) => {
            if resp.status() == 200 {
                let etag = resp.header("ETag").map(|s| s.to_string());
                let value: serde_json::Value =
                    resp.into_json().map_err(|e| format!("NETWORK {}", e))?;
                Ok((value, etag))
            } else {
                Err(format!("HTTP_{}", resp.status()))
            }
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP_{}", code)),
        Err(e) => Err(format!("NETWORK {}", e)),
    }
}

// ============ GitHub Device Flow（浏览器授权登录） ============

/// 共享版内置 OAuth App Client ID（公开值，可安全分发；config.oauth_client_id 可覆盖）
const DEFAULT_OAUTH_CLIENT_ID: &str = "Ov23liDmubrcYlV1vUGF";

fn github_form_post_once(
    a: &ureq::Agent,
    url: &str,
    form: &[(&str, &str)],
) -> Result<serde_json::Value, String> {
    let resp = a
        .post(url)
        .set("Accept", "application/json")
        .set("User-Agent", "Glint/1.0")
        .send_form(form)
        .map_err(|e| match e {
            ureq::Error::Status(c, _) => format!("HTTP_{}", c),
            e => format!("NETWORK {}", e),
        })?;
    resp.into_json().map_err(|e| format!("NETWORK {}", e))
}

fn github_form_post(url: &str, form: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let mut result = github_form_post_once(agent(), url, form);
    if matches!(&result, Err(e) if e.starts_with("NETWORK")) {
        if let Some(fb) = fallback_agent() {
            result = github_form_post_once(fb, url, form);
        }
    }
    result
}

#[derive(Serialize, Clone)]
struct DeviceCodeInfo {
    user_code: String,
    verification_url: String,
}

#[derive(Serialize, Clone)]
struct DeviceFlowResult {
    ok: bool,
    error: Option<String>, // cancelled / expired / denied / keyring_failed / bad_response / HTTP_xxx / NETWORK ...
}

/// 用户可监控的仓库（/user/repos，已过滤 fork 与归档仓库）
#[derive(Serialize, Clone)]
struct RepoBrief {
    full_name: String,
    private: bool,
    description: Option<String>,
    pushed_at: Option<String>,
}

/// 拉取单页 /user/repos（已过滤 fork 与归档仓库），供并发翻页复用
fn fetch_user_repos_page(token: &str, page: u32) -> Result<Vec<RepoBrief>, String> {
    let page_str = page.to_string();
    let (v, _) = get_json(
        &gh_url(
            "/user/repos",
            &[("per_page", "100"), ("page", page_str.as_str()), ("sort", "pushed")],
        ),
        Some(token),
    )?;
    let arr = v.as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();
    for r in arr {
        let full = r["full_name"].as_str().unwrap_or("").to_string();
        if full.is_empty() || r["archived"].as_bool().unwrap_or(false) || r["fork"].as_bool().unwrap_or(false) {
            continue;
        }
        out.push(RepoBrief {
            full_name: full,
            private: r["private"].as_bool().unwrap_or(false),
            description: r["description"].as_str().map(String::from),
            pushed_at: r["pushed_at"].as_str().map(String::from),
        });
    }
    Ok(out)
}

/// 同步命令跑在主线程会卡住整个 UI，凡阻塞网络 IO 一律丢到后台线程
#[tauri::command]
async fn fetch_user_repos() -> Result<Vec<RepoBrief>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let token = resolve_token().ok_or("NO_TOKEN")?;
        // 各页并发拉取、按页序合并（保持 pushed 排序），最慢一页决定总耗时而非逐页累加
        const MAX_PAGES: u32 = 5;
        let results: Vec<Result<Vec<RepoBrief>, String>> = std::thread::scope(|s| {
            let handles: Vec<_> = (1..=MAX_PAGES)
                .map(|page| {
                    let token_ref = &token;
                    s.spawn(move || fetch_user_repos_page(token_ref, page))
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap_or_else(|_| Err(String::from("THREAD"))))
                .collect()
        });
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for page_repos in results {
            let page_repos = page_repos?;
            if page_repos.is_empty() {
                break;
            }
            for repo in page_repos {
                if seen.insert(repo.full_name.clone()) {
                    out.push(repo);
                }
                if out.len() >= 500 {
                    break;
                }
            }
            if out.len() >= 500 {
                break;
            }
        }
        Ok(out)
    })
    .await
    .map_err(|e| format!("JOIN {}", e))?
}

/// 拉取失败 job 的日志并提取错误摘录——「失败后不打开网页、直接复制错误」的数据源
#[tauri::command]
async fn fetch_job_log_excerpt(repo: String, job_id: i64) -> Result<String, String> {
    if job_id <= 0 {
        return Err("NO_JOB".into());
    }
    // 阻塞网络 IO 放后台线程，避免卡住主线程
    tauri::async_runtime::spawn_blocking(move || {
        let token = resolve_token();
        let url = gh_url(
            &format!("/repos/{}/actions/jobs/{}/logs", repo, job_id),
            &[],
        );
        let excerpt = get_text(&url, token.as_deref()).and_then(|log| {
            let ex = extract_error_excerpt(&log, 120, 8000);
            if ex.trim().is_empty() {
                Err("EMPTY_LOG".into())
            } else {
                Ok(ex)
            }
        })?;
        Ok(excerpt)
    })
    .await
    .map_err(|e| format!("JOIN {}", e))?
}

#[tauri::command]
async fn device_flow_start(
    app: AppHandle,
    state: tauri::State<'_, Shared>,
    client_id: String,
) -> Result<DeviceCodeInfo, String> {
    let cid = {
        let trimmed = client_id.trim();
        if trimmed.is_empty() {
            DEFAULT_OAUTH_CLIENT_ID.to_string()
        } else {
            trimmed.to_string()
        }
    };
    if cid.is_empty() {
        return Err("NO_CLIENT_ID".into());
    }

    // 新会话会取消旧会话
    let cancel = {
        let mut slot = state.device_flow_cancel.lock().unwrap();
        if let Some(prev) = slot.replace(Arc::new(AtomicBool::new(false))) {
            prev.store(true, Ordering::Relaxed);
        }
        slot.as_ref().unwrap().clone()
    };

    // 阻塞网络 IO 丢到后台线程：同步命令跑在主线程会冻住整个 UI
    let cid_for_req = cid.clone();
    let info: serde_json::Value = tauri::async_runtime::spawn_blocking(move || {
        github_form_post(
            "https://github.com/login/device/code",
            &[("client_id", cid_for_req.as_str()), ("scope", "repo")],
        )
    })
    .await
    .map_err(|e| format!("JOIN {}", e))??;
    let device_code = info["device_code"].as_str().ok_or("BAD_RESPONSE")?.to_string();
    let user_code = info["user_code"].as_str().ok_or("BAD_RESPONSE")?.to_string();
    let verification_url = info["verification_uri_complete"]
        .as_str()
        .or_else(|| info["verification_uri"].as_str())
        .unwrap_or("https://github.com/login/device")
        .to_string();
    let poll_interval = info["interval"].as_u64().unwrap_or(5).max(3);
    eprintln!("[device-flow] device code obtained, opening browser, poll every {}s", poll_interval);
    open_external(&verification_url);

    let app2 = app.clone();
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        let mut interval = poll_interval;
        let app_for_finish = app2.clone();
        let finish = move |ok: bool, error: Option<&str>| {
            let _ = app_for_finish.emit(
                "device-flow",
                DeviceFlowResult {
                    ok,
                    error: error.map(String::from),
                },
            );
        };
        loop {
            if cancel.load(Ordering::Relaxed) {
                finish(false, Some("cancelled"));
                return;
            }
            std::thread::sleep(Duration::from_secs(interval));
            if cancel.load(Ordering::Relaxed) {
                finish(false, Some("cancelled"));
                return;
            }
            if started.elapsed() > Duration::from_secs(15 * 60) {
                finish(false, Some("expired"));
                return;
            }
            let form = [
                ("client_id", cid.as_str()),
                ("device_code", device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ];
            match github_form_post("https://github.com/login/oauth/access_token", &form) {
                Ok(v) => {
                    if let Some(token) = v["access_token"].as_str() {
                        eprintln!("[device-flow] access_token received, writing keychain");
                        let entry = keyring_entry();
                        let saved = match entry {
                            Some(e) => e.set_password(token).map_err(|err| err.to_string()),
                            None => Err("keyring unavailable".into()),
                        };
                        if let Err(err) = &saved {
                            // 不能吞掉错误：钥匙串写入失败要能从日志定位
                            eprintln!("[device-flow] keychain write FAILED: {err}");
                        }
                        if saved.is_ok() {
                            eprintln!("[device-flow] login complete");
                            finish(true, None);
                            wake(&app2);
                        } else {
                            finish(false, Some("keyring_failed"));
                        }
                        return;
                    }
                    match v["error"].as_str().unwrap_or("") {
                        "authorization_pending" => {}
                        "slow_down" => interval += 5,
                        "expired_token" => {
                            eprintln!("[device-flow] expired_token");
                            finish(false, Some("expired"));
                            return;
                        }
                        "access_denied" => {
                            eprintln!("[device-flow] access_denied");
                            finish(false, Some("denied"));
                            return;
                        }
                        "" => {
                            eprintln!("[device-flow] bad_response: {}", v.to_string().chars().take(300).collect::<String>());
                            finish(false, Some("bad_response"));
                            return;
                        }
                        other => {
                            eprintln!("[device-flow] github error: {}", other);
                            finish(false, Some(other));
                            return;
                        }
                    }
                }
                Err(e) => {
                    // 网络抖动：继续重试直到超时/取消；但必须留痕，否则表现为「没反应」
                    eprintln!("[device-flow] poll network error, will retry: {}", e);
                }
            }
        }
    });

    Ok(DeviceCodeInfo {
        user_code,
        verification_url,
    })
}

#[tauri::command]
fn device_flow_abort(state: tauri::State<Shared>) {
    if let Some(flag) = state.device_flow_cancel.lock().unwrap().as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
}

fn encode_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn gh_url(path: &str, query: &[(&str, &str)]) -> String {
    let mut url = format!("https://api.github.com{}", path);
    if !query.is_empty() {
        let qs: Vec<String> = query
            .iter()
            .map(|(k, v)| format!("{}={}", k, encode_component(v)))
            .collect();
        url.push('?');
        url.push_str(&qs.join("&"));
    }
    url
}

/// 返回 (JSON 值, 是否来自 304 缓存)。错误值为 key：HTTP_401 / HTTP_403 / HTTP_404 / NETWORK ...
fn get_json(url: &str, token: Option<&str>) -> Result<(serde_json::Value, bool), String> {
    let etag = etag_cache()
        .lock()
        .unwrap()
        .get(url)
        .map(|(e, _)| e.clone());

    // 主路由网络失败或被限流（代理出口 IP 共享未认证限额，本机直连往往还有余量）时切回退路由重试
    let mut result = get_json_once(agent(), url, token, etag.as_deref());
    if matches!(&result, Err(e) if e.starts_with("NETWORK") || e == "HTTP_403" || e == "HTTP_429") {
        if let Some(fb) = fallback_agent() {
            eprintln!("[net] primary route failed ({result:?}), retrying via fallback");
            result = get_json_once(fb, url, token, etag.as_deref());
        }
    }

    match result {
        Ok((value, new_etag)) => {
            if let Some(e) = new_etag {
                etag_cache()
                    .lock()
                    .unwrap()
                    .insert(url.to_string(), (e, value.clone()));
            }
            Ok((value, false))
        }
        Err(e) if e == "HTTP_304" => {
            let cache = etag_cache();
            if let Some((_, v)) = cache.lock().unwrap().get(url) {
                Ok((v.clone(), true))
            } else {
                Err("HTTP_304".into())
            }
        }
        Err(e) => Err(e),
    }
}

/// 拉取文本响应（日志端点返回 302 到签名 URL，ureq 默认跟随重定向，无需再带鉴权）
fn get_text_once(a: &ureq::Agent, url: &str, token: Option<&str>) -> Result<String, String> {
    let mut req = a
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", "Glint/1.0");
    if let Some(t) = token {
        req = req.set("Authorization", &format!("Bearer {}", t));
    }
    match req.call() {
        Ok(resp) => {
            if resp.status() == 200 {
                resp.into_string().map_err(|e| format!("NETWORK {}", e))
            } else {
                Err(format!("HTTP_{}", resp.status()))
            }
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP_{}", code)),
        Err(e) => Err(format!("NETWORK {}", e)),
    }
}

/// get_text：与 get_json 相同的双路由策略（主路由网络失败/限流时切回退路由重试）
fn get_text(url: &str, token: Option<&str>) -> Result<String, String> {
    let mut result = get_text_once(agent(), url, token);
    if matches!(&result, Err(e) if e.starts_with("NETWORK") || e == "HTTP_403" || e == "HTTP_429") {
        if let Some(fb) = fallback_agent() {
            eprintln!("[net] primary route failed ({result:?}), retrying via fallback");
            result = get_text_once(fb, url, token);
        }
    }
    result
}

/// 去掉 GitHub 日志行首的 ISO8601 时间戳前缀（如 `2026-11-03T01:02:03.1234567Z `）
fn strip_log_ts(line: &str) -> &str {
    let b = line.as_bytes();
    if b.len() > 28 && b[4] == b'-' && b[10] == b'T' {
        let end = 40.min(line.len());
        if let Some(p) = line[11..end].find("Z ") {
            return &line[p + 13..];
        }
    }
    line
}

/// 从 job 日志原文提取错误摘录：优先 `##[error]` 标记行（去时间戳），否则取末尾若干行
/// （许多工具不在注解里标 error，但失败原因总在日志尾部）。行数与字符数双重封顶。
fn extract_error_excerpt(log: &str, max_lines: usize, max_chars: usize) -> String {
    let clean = |l: &str| -> String {
        let mut s = strip_log_ts(l).to_string();
        // 剥离行首的 GitHub Actions 日志标记（##[error]/##[group]/##[endgroup] 等）
        if let Some(rest) = s.strip_prefix("##[") {
            if let Some(idx) = rest.find(']') {
                s = rest[idx + 1..].to_string();
            }
        }
        s.trim_end().to_string()
    };
    let lines: Vec<&str> = log.lines().collect();
    let err_idx: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("##[error]"))
        .map(|(i, _)| i)
        .collect();

    // 每个错误行向上带 15 行上下文、向下带 5 行（错误前执行的输出往往才是根因所在）；
    // 区间合并后顺序展开，空行不保留
    const BEFORE: usize = 15;
    const AFTER: usize = 5;
    let mut keep = vec![false; lines.len()];
    if err_idx.is_empty() {
        // 没有 ##[error] 标记：回退取日志末尾
        let start = lines.len().saturating_sub(max_lines);
        for k in start..lines.len() {
            keep[k] = true;
        }
    } else {
        for &i in &err_idx {
            let s = i.saturating_sub(BEFORE);
            let e = (i + AFTER + 1).min(lines.len());
            for k in s..e {
                keep[k] = true;
            }
        }
    }
    let mut picked: Vec<String> = Vec::new();
    let mut prev_kept = false;
    for (i, l) in lines.iter().enumerate() {
        if keep[i] {
            if !prev_kept && !picked.is_empty() {
                picked.push("……".to_string()); // 上下文区间断开处
            }
            picked.push(clean(l));
            prev_kept = true;
        } else {
            prev_kept = false;
        }
    }
    let picked: Vec<String> = picked
        .into_iter()
        .filter(|l| !l.trim().is_empty())
        .collect();

    let total = picked.len();
    let start = total.saturating_sub(max_lines);
    let mut out = String::new();
    if start > 0 {
        out.push_str("（日志过长，已截取末尾）\n");
    }
    for l in picked.into_iter().skip(start) {
        if out.len() + l.len() + 2 > max_chars {
            out.push_str("……\n");
            break;
        }
        out.push_str(&l);
        out.push('\n');
    }
    out.trim_end().to_string()
}

// ============ 状态判定（与 macOS 原生版一致） ============

fn is_failed_conclusion(c: Option<&str>) -> bool {
    // nil（无结论）视为失败，用于已完成的 check run
    matches!(
        c,
        Some("failure") | Some("timed_out") | Some("action_required") | Some("startup_failure") | None
    )
}

fn is_failure_conclusion(c: Option<&str>) -> bool {
    // 严格版：用于筛选已结束的 workflow run / job
    matches!(
        c,
        Some("failure") | Some("timed_out") | Some("action_required") | Some("startup_failure")
    )
}

fn aggregate_check_runs(runs: &[CheckRun]) -> &'static str {
    if runs.is_empty() {
        return "no_checks";
    }
    if runs.iter().any(|r| r.status != "completed") {
        return "running";
    }
    if runs
        .iter()
        .any(|r| r.status == "completed" && is_failed_conclusion(r.conclusion.as_deref()))
    {
        return "failed";
    }
    "success"
}

fn map_run_state(status: Option<&str>, conclusion: Option<&str>) -> String {
    match conclusion {
        Some("success") => "success".into(),
        Some("failure") | Some("timed_out") | Some("action_required") | Some("startup_failure") => {
            "failed".into()
        }
        Some("cancelled") => "cancelled".into(),
        Some("skipped") | Some("neutral") => "skipped".into(),
        Some(_) => "other".into(),
        None => match status {
            Some("queued") | Some("in_progress") | Some("waiting") | Some("pending") => "running".into(),
            _ => "other".into(),
        },
    }
}

fn aggregate_status(states: &[RepoState]) -> (&'static str, i64) {
    let mut failed = 0i64;
    let mut running = false;
    let mut success = false;
    let mut errored = false;
    for s in states {
        match s.status.as_str() {
            "failed" => failed += 1,
            "running" => running = true,
            "success" => success = true,
            "error" => errored = true,
            _ => {}
        }
    }
    if failed > 0 {
        ("failed", failed)
    } else if running {
        ("running", 0)
    } else if errored && !success {
        ("error", 0)
    } else if success {
        ("success", 0)
    } else {
        ("idle", 0)
    }
}

// ============ 拉取单个仓库 ============

fn fetch_repo(repo: &str, token: Option<&str>, light: bool) -> RepoState {
    let mut st = RepoState {
        repo: repo.to_string(),
        status: "loading".into(),
        open_prs: -1,
        open_issues: -1,
        ..Default::default()
    };
    if let Err(e) = fetch_repo_inner(repo, token, light, &mut st) {
        st.status = "error".into();
        st.error_text = Some(e);
    }
    st
}

// 从 /issues 响应提取面板用的 issue 摘要：过滤混入的 PR，按最近活跃排序，取前 8 条。
// 返回 (开放 issue 总数, 前 8 条摘要)。
fn parse_issue_briefs(v: &serde_json::Value) -> (i64, Vec<IssueBrief>) {
    let mut items: Vec<&serde_json::Value> = v
        .as_array()
        .map(|a| a.iter().filter(|i| i["pull_request"].is_null()).collect())
        .unwrap_or_default();
    let total = items.len() as i64;
    // RFC3339 同格式字符串可直接字典序比较
    items.sort_by(|a, b| {
        let ka = a["updated_at"].as_str().unwrap_or("");
        let kb = b["updated_at"].as_str().unwrap_or("");
        kb.cmp(ka)
    });
    let briefs = items
        .iter()
        .take(8)
        .map(|i| IssueBrief {
            number: i["number"].as_i64().unwrap_or(0),
            title: i["title"].as_str().unwrap_or("").to_string(),
            labels: i["labels"]
                .as_array()
                .map(|ls| {
                    ls.iter()
                        .map(|l| IssueLabel {
                            name: l["name"].as_str().unwrap_or("").to_string(),
                            color: l["color"].as_str().unwrap_or("").to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            url: i["html_url"].as_str().map(String::from),
            created_at: i["created_at"].as_str().map(String::from),
            updated_at: i["updated_at"].as_str().map(String::from),
            comments: i["comments"].as_i64().unwrap_or(0),
        })
        .collect();
    (total, briefs)
}

fn fetch_repo_inner(repo: &str, token: Option<&str>, light: bool, st: &mut RepoState) -> Result<(), String> {
    let meta = get_json(&gh_url(&format!("/repos/{}", repo), &[]), token)?.0;
    let branch = meta["default_branch"].as_str().unwrap_or("main").to_string();
    st.branch = Some(branch.clone());
    st.repo_url = meta["html_url"].as_str().map(String::from);
    st.stars = meta["stargazers_count"].as_i64().unwrap_or(0);
    st.pushed_at = meta["pushed_at"].as_str().map(String::from);

    let ref_enc = encode_component(&branch);
    let (crv, _) = get_json(
        &gh_url(
            &format!("/repos/{}/commits/{}/check-runs", repo, ref_enc),
            &[("per_page", "100")],
        ),
        token,
    )?;
    let total = crv["total_count"].as_i64().unwrap_or(0);
    let runs: Vec<CheckRun> = serde_json::from_value(crv["check_runs"].clone()).unwrap_or_default();

    if total > 0 {
        st.status = aggregate_check_runs(&runs).into();
        st.head_sha = runs.first().and_then(|r| r.head_sha.clone());
        let failed: Vec<&CheckRun> = runs
            .iter()
            .filter(|r| r.status == "completed" && is_failed_conclusion(r.conclusion.as_deref()))
            .collect();
        st.failed_count = failed.len() as i64;
        st.top_fail_url = failed.first().and_then(|r| r.html_url.clone());
        st.failed_checks = failed
            .iter()
            .take(8)
            .map(|r| FailedCheck {
                id: r.id,
                name: r.name.clone(),
                url: r.html_url.clone(),
                started_at: r.started_at.clone(),
                completed_at: r.completed_at.clone(),
            })
            .collect();
        st.total_checks = total;
        st.passed_checks = runs
            .iter()
            .filter(|r| matches!(r.conclusion.as_deref(), Some("success") | Some("neutral")))
            .count() as i64;
        st.skipped_checks = runs
            .iter()
            .filter(|r| r.conclusion.as_deref() == Some("skipped"))
            .count() as i64;
    } else {
        // 未使用 Check Runs 的仓库（纯 Status API 第三方 CI）
        let (cv, _) = get_json(
            &gh_url(&format!("/repos/{}/commits/{}/status", repo, ref_enc), &[]),
            token,
        )?;
        st.status = match cv["state"].as_str().unwrap_or("") {
            "success" => "success",
            "failure" | "error" => "failed",
            "pending" => "running",
            _ => "no_checks",
        }
        .into();
    }

    // 最近 Workflow 运行（用于面板展示），并记录最近一次失败的 run id
    let mut failed_run_id: Option<i64> = None;
    if let Ok((wv, _)) = get_json(
        &gh_url(
            &format!("/repos/{}/actions/runs", repo),
            &[("branch", &branch), ("per_page", "6")],
        ),
        token,
    ) {
        if let Some(arr) = wv["workflow_runs"].as_array() {
            for w in arr {
                let id = w["id"].as_i64().unwrap_or(0);
                let conclusion = w["conclusion"].as_str();
                if failed_run_id.is_none() && is_failure_conclusion(conclusion) {
                    failed_run_id = Some(id);
                }
                st.rows.push(RunRow {
                    id,
                    title: w["display_title"]
                        .as_str()
                        .or_else(|| w["name"].as_str())
                        .unwrap_or("workflow")
                        .to_string(),
                    state: map_run_state(w["status"].as_str(), conclusion),
                    url: w["html_url"].as_str().map(String::from),
                    date: w["created_at"].as_str().map(String::from),
                });
            }
        }
    }
    if st.rows.is_empty() && !runs.is_empty() {
        for r in runs.iter().take(6) {
            st.rows.push(RunRow {
                id: r.id,
                title: r.name.clone(),
                state: map_run_state(Some(&r.status), r.conclusion.as_deref()),
                url: r.html_url.clone(),
                date: r.completed_at.clone().or_else(|| r.started_at.clone()),
            });
        }
    }
    st.commit_subject = st.rows.first().map(|r| r.title.clone());

    // 失败时定位具体 job / step
    if st.status == "failed" {
        if let Some(run_id) = failed_run_id {
            if let Ok((jv, _)) = get_json(
                &gh_url(
                    &format!("/repos/{}/actions/runs/{}/jobs", repo, run_id),
                    &[("per_page", "100")],
                ),
                token,
            ) {
                if let Some(arr) = jv["jobs"].as_array() {
                    for j in arr
                        .iter()
                        .filter(|j| is_failure_conclusion(j["conclusion"].as_str()))
                        .take(6)
                    {
                        let failed_step = j["steps"]
                            .as_array()
                            .and_then(|steps| {
                                steps
                                    .iter()
                                    .find(|s| is_failure_conclusion(s["conclusion"].as_str()))
                                    .and_then(|s| s["name"].as_str().map(String::from))
                            });
                        st.failed_jobs.push(FailedJob {
                            id: j["id"].as_i64().unwrap_or(0),
                            name: j["name"].as_str().unwrap_or("").to_string(),
                            failed_step,
                            url: j["html_url"].as_str().map(String::from),
                            started_at: j["started_at"].as_str().map(String::from),
                            completed_at: j["completed_at"].as_str().map(String::from),
                        });
                    }
                }
            }
        }
    }

    if !light {
        if let Ok((pv, _)) = get_json(
            &gh_url(&format!("/repos/{}/pulls", repo), &[("state", "open"), ("per_page", "100")]),
            token,
        ) {
            st.open_prs = pv.as_array().map(|a| a.len() as i64).unwrap_or(0);
        }
        if let Ok((iv, _)) = get_json(
            &gh_url(&format!("/repos/{}/issues", repo), &[("state", "open"), ("per_page", "100")]),
            token,
        ) {
            let (total, briefs) = parse_issue_briefs(&iv);
            st.open_issues = total;
            st.issues = briefs;
        }
    }
    Ok(())
}

// ============ 托盘 ============

fn status_color(status: &str) -> (u8, u8, u8) {
    match status {
        "success" => (40, 205, 65),
        "failed" => (255, 59, 48),
        "running" => (255, 149, 0),
        "error" => (175, 82, 222),
        _ => (142, 142, 147),
    }
}

/// 程序生成的托盘图标：实心色块 + 镂空 git 分支 + 状态角标（RGBA）。
/// 色块颜色随菜单栏实际样式（macOS 26+ 恒白；旧版按主题深浅切换），无需外部资源。
fn tray_image(app: &AppHandle, r: u8, g: u8, b: u8) -> tauri::image::Image<'static> {
    let buf = tray_icon::render_tray_rgba(r, g, b, 32, menu_bar_is_dark(app));
    tauri::image::Image::new_owned(buf, 32, 32)
}

/// 菜单栏深浅判定：
/// - macOS 26+（Liquid Glass）：菜单栏不随系统深浅色翻转，系统图标恒白，
///   我们也恒白——跟 theme() 走反而会在浅色模式里变成一排白图标中的深色异类；
/// - 旧版 macOS：菜单栏跟随主题，用隐藏面板窗口的 theme()（effectiveAppearance）探测；
/// - 探测失败（个别 Linux 桌面环境）按深色菜单栏处理，维持白色块。
fn menu_bar_is_dark(app: &AppHandle) -> bool {
    #[cfg(target_os = "macos")]
    if macos_major() >= 26 {
        return true;
    }
    app.get_webview_window("panel")
        .and_then(|w| w.theme().ok())
        .map(|t| t == tauri::Theme::Dark)
        .unwrap_or(true)
}

/// macOS 产品版本主版本号（kern.osproductversion，如 27.1.0 → 27）；读取失败返回 0。
/// 只调用一次 sysctl 并缓存。
#[cfg(target_os = "macos")]
fn macos_major() -> u32 {
    static MAJOR: OnceLock<u32> = OnceLock::new();
    *MAJOR.get_or_init(|| {
        std::process::Command::new("sysctl")
            .args(["-n", "kern.osproductversion"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().split('.').next()?.parse().ok())
            .unwrap_or(0)
    })
}

/// 面板定位到托盘图标下方居中（macOS 版）。
/// 双屏坑：tray-icon 在 macOS 上报的 position/rect 是「全局屏幕逻辑坐标 × 托盘所在屏
/// scale」的伪物理坐标（tray-icon macos 源码用 NSEvent 坐标 × status 按钮 window 的
/// backingScaleFactor），而 tao 的 set_position(PhysicalPosition) 又按窗口当前所在屏
/// 的 scale 反推逻辑点——两屏缩放不同时两把尺子不一致，副屏点击后面板错位。
/// 因此先把坐标还原为全局逻辑点（points），再用 LogicalPosition 设置（tao 对
/// Logical 原样透传给 setFrameTopLeftPoint，不经过窗口当前屏换算），并夹进
/// 托盘所在显示器的可见范围。
#[cfg(target_os = "macos")]
fn position_panel_below_tray(
    w: &tauri::WebviewWindow,
    click: PhysicalPosition<f64>,
    rect: tauri::Rect,
) {
    // 还原 scale 的依据：托盘坐标 = 逻辑坐标 × 托盘屏 scale，故该点必落在
    // 那块屏「自身物理尺寸」表达的范围内
    let mon = w
        .available_monitors()
        .ok()
        .and_then(|ms| {
            ms.into_iter().find(|m| {
                let (mp, ms) = (m.position(), m.size());
                click.x >= mp.x as f64
                    && click.x < (mp.x + ms.width as i32) as f64
                    && click.y >= mp.y as f64
                    && click.y < (mp.y + ms.height as i32) as f64
            })
        })
        .or_else(|| w.primary_monitor().ok().flatten());
    let Some(mon) = mon else { return };

    let k = mon.scale_factor().max(0.0001);
    let (mx, my, mw, mh) = (
        mon.position().x as f64 / k,
        mon.position().y as f64 / k,
        mon.size().width as f64 / k,
        mon.size().height as f64 / k,
    );

    // 面板逻辑尺寸 = outer_size（窗口当前屏物理像素）÷ 窗口当前 scale
    let wk = w.scale_factor().unwrap_or(1.0).max(0.0001);
    let (pw, ph) = match w.outer_size() {
        Ok(s) => (s.width as f64 / wk, s.height as f64 / wk),
        Err(_) => (390.0, 560.0),
    };

    // 托盘图标中心 x / 底边 y 还原为逻辑点，面板置于其下方居中，四周留 4pt
    let rpos = rect.position.to_physical::<f64>(k);
    let rsize = rect.size.to_physical::<f64>(k);
    let icon_cx = (rpos.x + rsize.width / 2.0) / k;
    let icon_bottom = (rpos.y + rsize.height) / k;
    let x = (icon_cx - pw / 2.0).clamp(mx + 4.0, (mx + mw - pw - 4.0).max(mx + 4.0));
    let y = (icon_bottom + 6.0).clamp(my, (my + mh - ph).max(my));

    let _ = w.set_position(tauri::LogicalPosition::new(x, y));
}

/// 面板定位到托盘图标下方居中（Win/Linux 版）。
/// tray-icon 在这两端上报的是真实物理像素坐标（无 macOS 的 scale 伪坐标问题），
/// 直接 PhysicalPosition 定位即可。Windows 任务栏常驻屏幕底部：下方放不下则
/// 翻转到图标上方。Wayland 无全局坐标概念，合成器会忽略此定位（面板出现在
/// 默认位置，不 crash），X11 正常。
#[cfg(not(target_os = "macos"))]
fn position_panel_below_tray(
    w: &tauri::WebviewWindow,
    click: PhysicalPosition<f64>,
    rect: tauri::Rect,
) {
    let _ = click;
    let rpos = rect.position.to_physical::<f64>(1.0);
    let rsize = rect.size.to_physical::<f64>(1.0);
    let icon_cx = rpos.x + rsize.width / 2.0;
    let icon_bottom = rpos.y + rsize.height;

    // 面板物理尺寸 = outer_size（物理像素）
    let (pw, ph) = match w.outer_size() {
        Ok(s) => (s.width as f64, s.height as f64),
        Err(_) => (390.0, 560.0),
    };

    // 托盘图标中心所在的显示器；找不到回退主屏
    let mon = w
        .available_monitors()
        .ok()
        .and_then(|ms| {
            ms.into_iter().find(|m| {
                let (mp, msz) = (m.position(), m.size());
                let (cx, cy) = (rpos.x + rsize.width / 2.0, rpos.y + rsize.height / 2.0);
                cx >= mp.x as f64
                    && cx < (mp.x + msz.width as i32) as f64
                    && cy >= mp.y as f64
                    && cy < (mp.y + msz.height as i32) as f64
            })
        })
        .or_else(|| w.primary_monitor().ok().flatten());
    let Some(mon) = mon else { return };
    let (mx, my, mw, mh) = (
        mon.position().x as f64,
        mon.position().y as f64,
        mon.size().width as f64,
        mon.size().height as f64,
    );

    let x = (icon_cx - pw / 2.0).clamp(mx + 4.0, (mx + mw - pw - 4.0).max(mx + 4.0));
    let mut y = icon_bottom + 6.0;
    if y + ph > my + mh {
        // 底部放不下（任务栏在下）：翻到图标上方
        y = (rpos.y - ph - 6.0).max(my);
    }
    let _ = w.set_position(tauri::PhysicalPosition::new(x.round() as i32, y.round() as i32));
}

fn build_menu(app: &AppHandle, states: &[RepoState], failed: i64) -> tauri::Result<Menu<tauri::Wry>> {
    let mut items: Vec<Box<dyn IsMenuItem<tauri::Wry>>> = Vec::new();
    let title = if failed > 0 {
        format!("Glint · ❌ {}", failed)
    } else {
        "Glint".to_string()
    };
    items.push(Box::new(MenuItem::with_id(app, "noop", &title, false, None::<&str>)?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    for s in states {
        let emoji = match s.status.as_str() {
            "success" => "✅",
            "failed" => "❌",
            "running" => "⏳",
            "error" => "⚠️",
            "loading" => "🔄",
            _ => "⚪",
        };
        let id = format!("open:{}", s.repo);
        let label = format!("{} {}", emoji, s.repo);
        items.push(Box::new(MenuItem::with_id(app, id, &label, true, None::<&str>)?));
    }
    if !states.is_empty() {
        items.push(Box::new(PredefinedMenuItem::separator(app)?));
    }
    items.push(Box::new(MenuItem::with_id(app, "panel", "面板 Panel", true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, "manage", "管理 Manage", true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, "refresh", "刷新 Refresh", true, None::<&str>)?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, "quit", "退出 Quit", true, None::<&str>)?));

    let refs: Vec<&dyn IsMenuItem<tauri::Wry>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &refs)
}

fn update_tray(app: &AppHandle, states: &[RepoState]) {
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };
    let (agg, failed) = aggregate_status(states);
    let (r, g, b) = status_color(agg);
    let _ = tray.set_icon(Some(tray_image(app, r, g, b)));
    if let Ok(menu) = build_menu(app, states, failed) {
        let _ = tray.set_menu(Some(menu));
    }
}

fn show_panel(app: &AppHandle, manage: bool) {
    if manage {
        let _ = app.emit("show-manage", ());
    }
    if let Some(w) = app.get_webview_window("panel") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn wake(app: &AppHandle) {
    if let Some(s) = app.try_state::<Shared>() {
        let _ = s.wake.lock().unwrap().send(());
    }
}

fn open_external(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().0.as_str();
    if let Some(repo) = id.strip_prefix("open:") {
        open_external(&format!("https://github.com/{}/actions", repo));
        return;
    }
    match id {
        "panel" => show_panel(app, false),
        "manage" => show_panel(app, true),
        "refresh" => wake(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============ 通知文案（按 config.lang，空则跟随系统语言） ============

/// 区域标识 → 通知语言：zh* 视为中文，其余一律英文（国际化默认；
/// 旧逻辑非英文即中文，会让其他语种用户收到中文通知）
fn locale_to_lang(locale: &str) -> String {
    let l = locale.trim().to_ascii_lowercase();
    if l.starts_with("zh") { "zh".into() } else { "en".into() }
}

fn detect_lang() -> String {
    #[cfg(target_os = "macos")]
    {
        // GUI 启动不继承 shell 的 LANG，读系统区域设置
        if let Ok(out) = std::process::Command::new("defaults").args(["read", "-g", "AppleLocale"]).output() {
            if out.status.success() {
                return locale_to_lang(&String::from_utf8_lossy(&out.stdout));
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        // Windows GUI 进程没有 LANG，读用户区域（注册表 LocaleName，如 zh-CN）；
        // reg query 输出形如 "    LocaleName    REG_SZ    zh-CN"，取末字段
        if let Ok(out) = std::process::Command::new("reg")
            .args(["query", r"HKCU\Control Panel\International", "/v", "LocaleName"])
            .output()
        {
            if out.status.success() {
                if let Some(loc) = String::from_utf8_lossy(&out.stdout).split_whitespace().last() {
                    return locale_to_lang(loc);
                }
            }
        }
    }
    if let Ok(l) = std::env::var("LANG") {
        return locale_to_lang(&l);
    }
    "en".into()
}

fn notif_lang(cfg_lang: &str) -> bool {
    // true = 英文
    let l = if cfg_lang.is_empty() {
        static SYS: OnceLock<String> = OnceLock::new();
        SYS.get_or_init(detect_lang).clone()
    } else {
        cfg_lang.to_string()
    };
    l == "en"
}

/// 通知插件桌面端无点击回调（Actions API 仅移动端）；点击通知由系统激活应用，
/// 配合 main() 里的 RunEvent::Reopen 唤起面板，失败仓库已置顶可直接看到详情。
fn notify_status_change(app: &AppHandle, cfg: &Config, st: &RepoState, old: &str) {
    let en = notif_lang(&cfg.lang);
    let (title, body): (String, String) = if st.status == "failed" && cfg.notify_on_failure {
        let title = if en {
            format!("❌ {} CI failed", st.repo)
        } else {
            format!("❌ {} CI 失败", st.repo)
        };
        let body = if st.failed_count > 0 {
            if en {
                if st.failed_count == 1 { "1 check failed".into() } else { format!("{} checks failed", st.failed_count) }
            } else {
                format!("{} 项检查未通过", st.failed_count)
            }
        } else if en {
            "Checks failed".into()
        } else {
            "检查失败".into()
        };
        (title, body)
    } else if st.status == "success" && cfg.notify_on_success && matches!(old, "running" | "failed" | "error") {
        let title = if en {
            format!("✅ {} CI passed", st.repo)
        } else {
            format!("✅ {} CI 通过", st.repo)
        };
        let body: String = if en { "All checks passed".into() } else { "全部检查通过".into() };
        (title, body)
    } else {
        return;
    };

    // macOS：原生 UNUserNotificationCenter（delegate 接管点击 → 唤起面板）
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        macos_notify::notify(app, &title, &body, &st.repo);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app.notification().builder().title(title).body(body).show();
    }
}

/// 失败 > 错误 > 运行中 > 其余；托盘菜单与面板用此优先级展示
fn severity_rank(status: &str) -> u8 {
    match status {
        "failed" => 0,
        "error" => 1,
        "running" => 2,
        _ => 3,
    }
}

// ============ 轮询线程 ============

fn poller(app: AppHandle, rx: mpsc::Receiver<()>) {
    let mut prev: HashMap<String, String> = HashMap::new();
    loop {
        let shared = app.state::<Shared>();
        shared.refreshing.store(true, Ordering::Relaxed);

        let cfg = load_config();
        let token = resolve_token();
        let light = token.is_none();
        let repos = cfg.repos.clone();

        let map: Arc<Mutex<HashMap<String, RepoState>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for repo in repos.clone() {
            let map = map.clone();
            let token = token.clone();
            handles.push(std::thread::spawn(move || {
                let st = fetch_repo(&repo, token.as_deref(), light);
                map.lock().unwrap().insert(repo, st);
            }));
        }
        for h in handles {
            let _ = h.join();
        }
        let mut states: Vec<RepoState> = repos
            .iter()
            .filter_map(|r| map.lock().unwrap().get(r).cloned())
            .collect();
        // 失败仓库置顶（稳定排序，组内保持配置顺序），托盘菜单与面板共用此顺序
        states.sort_by_key(|s| severity_rank(&s.status));

        // 状态变化通知
        for st in &states {
            if let Some(old) = prev.get(&st.repo) {
                if *old != st.status {
                    notify_status_change(&app, &cfg, st, old);
                }
            }
            prev.insert(st.repo.clone(), st.status.clone());
        }

        *shared.snapshot.lock().unwrap() = states.clone();
        let ts = now_ms();
        *shared.last_refresh_ms.lock().unwrap() = ts;
        shared.refreshing.store(false, Ordering::Relaxed);
        update_tray(&app, &states);
        let _ = app.emit(
            "status-updated",
            Snapshot { repos: states, last_refresh_ms: ts, refreshing: false },
        );

        let secs = if light {
            cfg.interval_seconds.max(240)
        } else {
            cfg.interval_seconds.max(10)
        };
        let _ = rx.recv_timeout(Duration::from_secs(secs));
    }
}

// ============ 本地目录扫描（可选功能） ============

fn extract_repo_from_line(line: &str) -> Option<String> {
    let idx = line.find("github.com")?;
    let rest = &line[idx + "github.com".len()..];
    let rest = rest.trim_start_matches(|c| c == ':' || c == '/');
    let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
    let mut repo = rest[..end].trim().to_string();
    if repo.ends_with(".git") {
        repo.truncate(repo.len() - 4);
    }
    while repo.ends_with('/') {
        repo.pop();
    }
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    if parts.next().is_some() || owner.is_empty() || name.is_empty() {
        return None;
    }
    let valid =
        |s: &str| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || "-_.".contains(c));
    if !valid(owner) || !valid(name) {
        return None;
    }
    Some(format!("{}/{}", owner, name))
}

fn scan_local_dir() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return vec![];
    };
    let mut found = std::collections::BTreeSet::new();
    for dir in ["code", "projects", "dev", "src", "work"] {
        let root = home.join(dir);
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            for repo_dir in [entry.path()] {
                // 第一层
                let cfg_path = repo_dir.join(".git/config");
                if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                    for line in content.lines() {
                        if line.contains("github.com") {
                            if let Some(r) = extract_repo_from_line(line) {
                                found.insert(r);
                            }
                        }
                    }
                }
                // 第二层
                if let Ok(subs) = std::fs::read_dir(&repo_dir) {
                    for sub in subs.flatten() {
                        let cfg_path = sub.path().join(".git/config");
                        if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                            for line in content.lines() {
                                if line.contains("github.com") {
                                    if let Some(r) = extract_repo_from_line(line) {
                                        found.insert(r);
                                    }
                                }
                            }
                        }
                    }
                }
                if found.len() >= 30 {
                    return found.into_iter().collect();
                }
            }
        }
    }
    found.into_iter().collect()
}

// ============ Tauri 命令 ============

#[tauri::command]
fn get_snapshot(app: AppHandle) -> Snapshot {
    let s = app.state::<Shared>();
    let snapshot = Snapshot {
        repos: s.snapshot.lock().unwrap().clone(),
        last_refresh_ms: *s.last_refresh_ms.lock().unwrap(),
        refreshing: s.refreshing.load(Ordering::Relaxed),
    };
    snapshot
}

#[tauri::command]
async fn get_config() -> Config {
    // 配置文件磁盘 IO 丢到后台线程，避免占住主线程
    tauri::async_runtime::spawn_blocking(load_config)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn save_config(app: AppHandle, config: Config) {
    let _ = tauri::async_runtime::spawn_blocking(move || save_config_file(&config)).await;
    wake(&app);
}

#[tauri::command]
fn refresh_now(app: AppHandle) {
    wake(&app);
}

#[tauri::command]
async fn token_info_cmd() -> TokenInfo {
    // 钥匙串读取是阻塞系统调用（ACL 未信任时还可能弹授权框），丢到后台线程
    tauri::async_runtime::spawn_blocking(token_info)
        .await
        .unwrap_or_else(|_| TokenInfo { present: false, source: "none".into() })
}

#[tauri::command]
async fn import_local_dir() -> Vec<String> {
    // 目录树扫描是阻塞文件 IO，丢到后台线程
    tauri::async_runtime::spawn_blocking(scan_local_dir)
        .await
        .unwrap_or_default()
}

#[tauri::command]
fn open_url(_app: AppHandle, url: String) {
    open_external(&url);
}

/// 「关于」区展示的当前版本号
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// 前端 JS 错误/打点上报，输出到 stderr 日志
#[tauri::command]
fn log_js(msg: String) {
    eprintln!("[js] {}", msg);
}

// ============ 检查更新（半自动：查 latest release，点击下载安装包） ============
// 完整静默更新（Tauri updater）需要 minisign 密钥 + Apple 签名以避免 macOS 换包被 Gatekeeper 拦，
// 签名就绪前用「横幅提示 + 打开下载」方案，零签名依赖。

#[derive(Serialize)]
struct UpdateInfo {
    has_update: bool,
    current: String,
    latest: String,
    url: String, // 安装包直链；找不到资产时退回 release 页
}

fn parse_ver(v: &str) -> (u32, u32, u32) {
    let nums: Vec<u32> = v
        .trim_start_matches('v')
        .split('.')
        .filter_map(|p| p.split('-').next().and_then(|s| s.parse().ok()))
        .collect();
    (
        nums.first().copied().unwrap_or(0),
        nums.get(1).copied().unwrap_or(0),
        nums.get(2).copied().unwrap_or(0),
    )
}

/// 已知架构关键词全集：用于识别「异架构」资产（如 Intel 机器撞上 arm64 专属包）
const ARCH_KEYWORDS: &[&str] = &[
    "x86_64", "x64", "amd64", "x86", "i686", "i386", "aarch64", "arm64", "armv7",
];

/// 从 Release 资产名中挑出当前平台应下载的安装包（返回下标）。
/// 纯函数、平台无关（平台参数由调用方以 cfg!(...) 传入），三端规则各自可单测：
/// - 扩展名必中，exts 顺序即优先级（Windows 下 NSIS 靠 -setup bonus 压过 msi）
/// - arch 命中加分；命中**别的**架构关键词直接排除（异架构包装上也没用）
/// - mac_fallback：全部被排除时接受第一个扩展名命中的资产——macOS universal 产物
///   固定命名 *_aarch64.dmg，Intel 机不能因此拿不到包退回 release 页
fn pick_update_asset(
    names: &[&str],
    exts: &[&str],
    archs: &[&str],
    mac_fallback: bool,
) -> Option<usize> {
    let lower: Vec<String> = names.iter().map(|n| n.to_ascii_lowercase()).collect();
    let mut best: Option<(i32, usize)> = None;
    for (i, name) in lower.iter().enumerate() {
        let Some(ext_rank) = exts.iter().position(|e| name.ends_with(e)) else {
            continue;
        };
        let hit = archs.iter().any(|a| name.contains(a));
        let wrong_arch = !hit
            && ARCH_KEYWORDS
                .iter()
                .any(|k| name.contains(k) && !archs.contains(k));
        if wrong_arch {
            continue;
        }
        let score = 10 - ext_rank as i32
            + i32::from(hit) * 2
            + i32::from(name.contains("-setup"));
        if best.map_or(true, |(s, _)| score > s) {
            best = Some((score, i));
        }
    }
    if best.is_none() && mac_fallback {
        return lower
            .iter()
            .position(|n| exts.iter().any(|e| n.ends_with(e)));
    }
    best.map(|(_, i)| i)
}

#[cfg(test)]
mod update_asset_tests {
    use super::pick_update_asset;

    // 用 Tauri 真实产物命名做三端匹配回归（大小写、arch 别名都要对得上）。
    // 注意 CI universal 构建的 DMG 名是 *_universal.dmg（本机单架构才是 *_aarch64.dmg）
    const MAC: &[&str] = &["Glint_1.1.0.app.tar.gz", "Glint_1.1.0_universal.dmg"];
    const WIN: &[&str] = &[
        "Glint_1.1.0_x64_en-US.msi",
        "Glint_1.1.0_x64-setup.exe",
        "glint_1.1.0_amd64.AppImage",
        "glint_1.1.0_amd64.deb",
        "Glint_1.1.0_x64-setup.exe.blockmap",
    ];
    const LIN: &[&str] = &[
        "glint_1.1.0_amd64.deb",
        "glint_1.1.0_amd64.AppImage",
        "Glint_1.1.0_x64_en-US.msi",
    ];

    #[test]
    fn mac_picks_dmg() {
        let i = pick_update_asset(MAC, &[".dmg"], &["aarch64", "arm64"], true).unwrap();
        assert!(MAC[i].ends_with(".dmg"), "picked {}", MAC[i]);
    }

    #[test]
    fn mac_intel_falls_back_to_universal_dmg() {
        // universal DMG 名字不含 arch 关键词：arm/Intel 都按纯扩展名匹配拿到包
        // （"universal" 不在架构关键词表里，不会被当异架构排除）
        let i = pick_update_asset(MAC, &[".dmg"], &["x64", "x86_64"], true).unwrap();
        assert!(MAC[i].ends_with(".dmg"), "picked {}", MAC[i]);
    }

    #[test]
    fn mac_prefers_arch_named_dmg_when_present() {
        // 若同名版本既有 universal 又有 aarch64 命名：arm 机优先精确命中，Intel 机
        // 排除异架构（aarch64 关键词）后拿 universal
        let both = ["Glint_1.1.0_universal.dmg", "Glint_1.1.0_aarch64.dmg"];
        let arm = pick_update_asset(&both, &[".dmg"], &["aarch64", "arm64"], true).unwrap();
        assert!(both[arm].contains("aarch64"), "picked {}", both[arm]);
        let intel = pick_update_asset(&both, &[".dmg"], &["x64", "x86_64"], true).unwrap();
        assert!(both[intel].contains("universal"), "picked {}", both[intel]);
    }

    #[test]
    fn windows_prefers_nsis_setup_exe_over_msi() {
        let i = pick_update_asset(WIN, &[".exe", ".msi"], &["x64", "amd64", "x86_64"], false)
            .unwrap();
        assert!(WIN[i].contains("-setup.exe"), "picked {}", WIN[i]);
    }

    #[test]
    fn linux_picks_appimage_ignoring_deb_and_msi() {
        let i = pick_update_asset(LIN, &[".appimage"], &["amd64", "x86_64", "x64"], false)
            .unwrap();
        assert!(LIN[i].to_ascii_lowercase().ends_with(".appimage"));
    }

    #[test]
    fn wrong_arch_asset_rejected() {
        let only_arm = ["Glint_1.1.0_arm64-setup.exe"];
        assert!(pick_update_asset(
            &only_arm,
            &[".exe", ".msi"],
            &["x64", "amd64", "x86_64"],
            false
        )
        .is_none());
    }
}

#[tauri::command]
async fn check_update() -> Result<Option<UpdateInfo>, String> {
    // 阻塞网络 IO 丢到后台线程：同步命令跑在主线程会冻住整个 UI
    tauri::async_runtime::spawn_blocking(|| {
        const OWNER_REPO: &str = "xinnix/glint";
        let current = env!("CARGO_PKG_VERSION");

        // 404（还没有正式 release）与网络失败都静默降级为「无更新」，不影响主功能
        let Ok((rel, _)) = get_json(
            &format!("https://api.github.com/repos/{}/releases/latest", OWNER_REPO),
            None,
        ) else {
            return Ok(None);
        };
        let Some(tag) = rel["tag_name"].as_str().filter(|t| !t.is_empty()) else {
            return Ok(None);
        };
        if parse_ver(tag) <= parse_ver(current) {
            return Ok(None);
        }

        // 按平台/架构挑安装包资产；挑不到就退回 release 页。
        // Tauri 实际产物命名（大小写、arch 别名都与直觉不同，勿按 x86_64 字面匹配）：
        // Glint_1.1.0_x64_en-US.msi / Glint_1.1.0_x64-setup.exe / glint_1.1.0_amd64.AppImage
        // （Linux 包名小写）/ Glint_1.1.0_aarch64.dmg（universal）
        let (exts, archs, mac_fallback) = if cfg!(target_os = "macos") {
            (&[".dmg"][..], &["aarch64", "arm64"][..], true)
        } else if cfg!(target_os = "windows") {
            (&[".exe", ".msi"][..], &["x64", "amd64", "x86_64"][..], false)
        } else {
            (&[".appimage"][..], &["amd64", "x86_64", "x64"][..], false)
        };
        let release_page = rel["html_url"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("https://github.com/{}/releases/latest", OWNER_REPO));
        let mut url = release_page.clone();
        if let Some(assets) = rel["assets"].as_array() {
            let names: Vec<&str> = assets.iter().filter_map(|a| a["name"].as_str()).collect();
            if let Some(i) = pick_update_asset(&names, exts, archs, mac_fallback) {
                if let Some(link) = assets[i]["browser_download_url"].as_str() {
                    url = link.to_string();
                }
            }
        }
        Ok(Some(UpdateInfo {
            has_update: true,
            current: current.to_string(),
            latest: tag.trim_start_matches('v').to_string(),
            url,
        }))
    })
    .await
    .map_err(|e| format!("JOIN {}", e))?
}

// ============ 开机自启（不引插件：LaunchAgent / 注册表 Run / XDG autostart） ============

#[tauri::command]
async fn autostart_state() -> bool {
    // Windows 走 reg 子进程查询；统一丢后台线程避免卡主线程
    tauri::async_runtime::spawn_blocking(autostart_enabled)
        .await
        .unwrap_or(false)
}

/// 写入/移除登录自启项；仅改文件，下次登录生效（不 load，避免运行中双实例）
#[tauri::command]
async fn set_autostart_cmd(enable: bool) -> Result<(), String> {
    // Windows 走 reg 子进程写入；统一丢后台线程避免卡主线程
    tauri::async_runtime::spawn_blocking(move || set_autostart(enable))
        .await
        .map_err(|e| format!("JOIN {}", e))?
}

#[cfg(target_os = "macos")]
fn autostart_file() -> std::path::PathBuf {
    let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push("Library/LaunchAgents/io.github.glint.plist");
    p
}

#[cfg(target_os = "macos")]
fn autostart_enabled() -> bool {
    autostart_file().exists()
}

#[cfg(target_os = "macos")]
fn set_autostart(enable: bool) -> Result<(), String> {
    let path = autostart_file();
    if !enable {
        let _ = std::fs::remove_file(&path);
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>io.github.glint</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>ProcessType</key><string>Interactive</string>
</dict>
</plist>
"#,
        exe.display()
    );
    std::fs::write(&path, plist).map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
const AUTOSTART_REG: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "windows")]
fn autostart_enabled() -> bool {
    std::process::Command::new("reg")
        .args(["query", AUTOSTART_REG, "/v", "Glint"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn set_autostart(enable: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if enable {
        let val = format!("\"{}\"", exe.display());
        std::process::Command::new("reg")
            .args(["add", AUTOSTART_REG, "/v", "Glint", "/t", "REG_SZ", "/d", &val, "/f"])
            .output()
            .map_err(|e| e.to_string())?;
    } else {
        std::process::Command::new("reg")
            .args(["delete", AUTOSTART_REG, "/v", "Glint", "/f"])
            .output()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn autostart_file() -> std::path::PathBuf {
    let mut p = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    p.push("autostart/glint.desktop");
    p
}

#[cfg(target_os = "linux")]
fn autostart_enabled() -> bool {
    autostart_file().exists()
}

#[cfg(target_os = "linux")]
fn set_autostart(enable: bool) -> Result<(), String> {
    let path = autostart_file();
    if !enable {
        let _ = std::fs::remove_file(&path);
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Glint\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    );
    std::fs::write(&path, entry).map_err(|e| e.to_string())
}

// ============ 入口 ============

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .on_window_event(|window, event| {
            if window.label() == "panel" {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
                // 系统深浅色切换：菜单栏外观跟着变，托盘色块颜色即时重绘
                if let tauri::WindowEvent::ThemeChanged(_) = event {
                    let app = window.app_handle();
                    let states = app
                        .try_state::<Shared>()
                        .map(|sh| sh.snapshot.lock().unwrap().clone())
                        .unwrap_or_default();
                    update_tray(app, &states);
                }
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            migrate_from_legacy();

            // macOS：装原生通知 delegate（必须在应用完成启动前，点击回调才生效）
            #[cfg(target_os = "macos")]
            macos_notify::init(app.handle().clone());

            let (tx, rx) = mpsc::channel::<()>();
            app.manage(Shared {
                snapshot: Mutex::new(Vec::new()),
                last_refresh_ms: Mutex::new(0),
                refreshing: AtomicBool::new(false),
                wake: Mutex::new(tx),
                device_flow_cancel: Mutex::new(None),
            });

            // 先建隐藏面板窗口：托盘色块颜色要用它的 theme()（effectiveAppearance）探测系统深浅
            WebviewWindowBuilder::new(app, "panel", WebviewUrl::App("index.html".into()))
                .title("Glint")
                .inner_size(400.0, 600.0)
                .decorations(false)
                .resizable(false)
                .visible(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .build()?;

            let menu = build_menu(app.handle(), &[], 0)?;
            TrayIconBuilder::with_id("main")
                .icon(tray_image(app.handle(), 142, 142, 147))
                .tooltip("Glint")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| handle_menu_event(app, event))
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("panel") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                position_panel_below_tray(&w, position, rect);
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            let handle = app.handle().clone();
            std::thread::spawn(move || poller(handle, rx));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_config,
            save_config,
            refresh_now,
            token_info_cmd,
            fetch_user_repos,
            fetch_job_log_excerpt,
            import_local_dir,
            open_url,
            quit_app,
            log_js,
            autostart_state,
            set_autostart_cmd,
            check_update,
            app_version,
            device_flow_start,
            device_flow_abort
        ])
        .build(tauri::generate_context!())
        .expect("Glint 启动失败")
        .run(|app, event| {
            // macOS：点击系统通知会激活应用并触发 reopen；此时唤起面板，
            // 失败仓库已置顶，一眼可见失败详情（通知插件桌面端无点击回调）。
            // RunEvent::Reopen 是 macOS 特有变体，非 macOS 无等价机制（README 已知限制）
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event {
                show_panel(app, false);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}
