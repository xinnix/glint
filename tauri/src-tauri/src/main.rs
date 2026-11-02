#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// GitCIStatus 跨平台版（Tauri）：托盘 + 面板监控 GitHub CI
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
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

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
    token: String,
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
            token: String::new(),
            notify_on_failure: true,
            notify_on_success: true,
            lang: String::new(),
            oauth_client_id: String::new(),
        }
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
}

#[derive(Debug, Clone, Serialize)]
struct TokenInfo {
    present: bool,
    source: String, // env / keychain / gh / config / none
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
    let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push(".config/gitcistatus");
    let _ = std::fs::create_dir_all(&p);
    p.push("config.json");
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

// ============ Token 解析：环境变量 → 钥匙串 → gh CLI → 配置文件 ============

fn run_tool(path: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(path).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

fn keyring_entry() -> Option<keyring::Entry> {
    keyring::Entry::new("gitcistatus", "token").ok()
}

fn resolve_token(cfg_token: &str) -> Option<String> {
    for key in ["GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(t) = std::env::var(key) {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    if let Some(entry) = keyring_entry() {
        if let Ok(t) = entry.get_password() {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    for path in ["/opt/homebrew/bin/gh", "/usr/local/bin/gh", "/usr/bin/gh", "gh"] {
        if let Some(t) = run_tool(path, &["auth", "token"]) {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    if !cfg_token.is_empty() {
        return Some(cfg_token.to_string());
    }
    None
}

fn token_info(cfg_token: &str) -> TokenInfo {
    for key in ["GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(t) = std::env::var(key) {
            if !t.is_empty() {
                return TokenInfo { present: true, source: "env".into() };
            }
        }
    }
    if let Some(entry) = keyring_entry() {
        if let Ok(t) = entry.get_password() {
            if !t.is_empty() {
                return TokenInfo { present: true, source: "keychain".into() };
            }
        }
    }
    for path in ["/opt/homebrew/bin/gh", "/usr/local/bin/gh", "/usr/bin/gh", "gh"] {
        if let Some(t) = run_tool(path, &["auth", "token"]) {
            if !t.is_empty() && !t.is_empty() {
                return TokenInfo { present: true, source: "gh".into() };
            }
        }
    }
    if !cfg_token.is_empty() {
        return TokenInfo { present: true, source: "config".into() };
    }
    TokenInfo { present: false, source: "none".into() }
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
        .set("User-Agent", "GitCIStatus/1.0");
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
        .set("User-Agent", "GitCIStatus/1.0")
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

#[tauri::command]
fn fetch_user_repos() -> Result<Vec<RepoBrief>, String> {
    let cfg = load_config();
    let token = resolve_token(&cfg.token).ok_or("NO_TOKEN")?;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for page in 1..=5u32 {
        let page_str = page.to_string();
        let (v, _) = get_json(
            &gh_url(
                "/user/repos",
                &[("per_page", "100"), ("page", page_str.as_str()), ("sort", "pushed")],
            ),
            Some(token.as_str()),
        )?;
        let arr = v.as_array().cloned().unwrap_or_default();
        if arr.is_empty() {
            break;
        }
        for r in arr {
            let full = r["full_name"].as_str().unwrap_or("").to_string();
            if full.is_empty() || !seen.insert(full.clone()) {
                continue;
            }
            if r["archived"].as_bool().unwrap_or(false) || r["fork"].as_bool().unwrap_or(false) {
                continue;
            }
            out.push(RepoBrief {
                full_name: full,
                private: r["private"].as_bool().unwrap_or(false),
                description: r["description"].as_str().map(String::from),
                pushed_at: r["pushed_at"].as_str().map(String::from),
            });
            if out.len() >= 500 {
                break;
            }
        }
        if out.len() >= 500 {
            break;
        }
    }
    Ok(out)
}

#[tauri::command]
fn device_flow_start(
    app: AppHandle,
    state: tauri::State<Shared>,
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

    let info: serde_json::Value = github_form_post(
        "https://github.com/login/device/code",
        &[("client_id", cid.as_str()), ("scope", "repo")],
    )?;
    let device_code = info["device_code"].as_str().ok_or("BAD_RESPONSE")?.to_string();
    let user_code = info["user_code"].as_str().ok_or("BAD_RESPONSE")?.to_string();
    let verification_url = info["verification_uri_complete"]
        .as_str()
        .or_else(|| info["verification_uri"].as_str())
        .unwrap_or("https://github.com/login/device")
        .to_string();
    let poll_interval = info["interval"].as_u64().unwrap_or(5).max(3);
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
                        let saved = keyring_entry()
                            .and_then(|e| e.set_password(token).ok())
                            .is_some();
                        if saved {
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
                            finish(false, Some("expired"));
                            return;
                        }
                        "access_denied" => {
                            finish(false, Some("denied"));
                            return;
                        }
                        "" => {
                            finish(false, Some("bad_response"));
                            return;
                        }
                        other => {
                            finish(false, Some(other));
                            return;
                        }
                    }
                }
                Err(_) => {} // 网络抖动：继续重试直到超时/取消
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

    // 主路由网络失败时自动切换回退路由重试
    let mut result = get_json_once(agent(), url, token, etag.as_deref());
    if matches!(&result, Err(e) if e.starts_with("NETWORK")) {
        if let Some(fb) = fallback_agent() {
            eprintln!("[net] primary route failed, retrying via fallback");
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
            st.open_issues = iv
                .as_array()
                .map(|a| a.iter().filter(|i| i["pull_request"].is_null()).count() as i64)
                .unwrap_or(0);
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

/// 程序生成的托盘图标：git 分支图形 + 状态角标（RGBA），无需外部资源
fn tray_image(r: u8, g: u8, b: u8) -> tauri::image::Image<'static> {
    let buf = tray_icon::render_tray_rgba(r, g, b, 32);
    tauri::image::Image::new_owned(buf, 32, 32)
}

fn build_menu(app: &AppHandle, states: &[RepoState], failed: i64) -> tauri::Result<Menu<tauri::Wry>> {
    let mut items: Vec<Box<dyn IsMenuItem<tauri::Wry>>> = Vec::new();
    let title = if failed > 0 {
        format!("GitCIStatus · ❌ {}", failed)
    } else {
        "GitCIStatus".to_string()
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
    let _ = tray.set_icon(Some(tray_image(r, g, b)));
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

// ============ 轮询线程 ============

fn poller(app: AppHandle, rx: mpsc::Receiver<()>) {
    let mut prev: HashMap<String, String> = HashMap::new();
    loop {
        let cycle_start = std::time::Instant::now();
        eprintln!("[poller] cycle start");
        let shared = app.state::<Shared>();
        shared.refreshing.store(true, Ordering::Relaxed);

        let cfg = load_config();
        let token = resolve_token(&cfg.token);
        eprintln!(
            "[poller] {} repos, token={}, light={}",
            cfg.repos.len(),
            if token.is_some() { "yes" } else { "no" },
            token.is_none()
        );
        let light = token.is_none();
        let repos = cfg.repos.clone();

        let map: Arc<Mutex<HashMap<String, RepoState>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for repo in repos.clone() {
            let map = map.clone();
            let token = token.clone();
            handles.push(std::thread::spawn(move || {
                let t0 = std::time::Instant::now();
                let st = fetch_repo(&repo, token.as_deref(), light);
                eprintln!(
                    "[fetch] {} -> {} err={:?} ({:?})",
                    repo,
                    st.status,
                    st.error_text,
                    t0.elapsed()
                );
                map.lock().unwrap().insert(repo, st);
            }));
        }
        for h in handles {
            let _ = h.join();
        }
        eprintln!("[poller] fetch phase done in {:?}", cycle_start.elapsed());
        let states: Vec<RepoState> = repos
            .iter()
            .filter_map(|r| map.lock().unwrap().get(r).cloned())
            .collect();

        // 状态变化通知
        for st in &states {
            let old = prev.get(&st.repo);
            if let Some(old) = old {
                if *old != st.status {
                    if st.status == "failed" && cfg.notify_on_failure {
                        let body = if st.failed_count > 0 {
                            format!("{} 项检查未通过 · {} check(s) failed", st.failed_count, st.failed_count)
                        } else {
                            "检查失败 · checks failed".to_string()
                        };
                        let _ = app
                            .notification()
                            .builder()
                            .title(format!("❌ {} CI 失败", st.repo))
                            .body(body)
                            .show();
                    } else if st.status == "success"
                        && cfg.notify_on_success
                        && matches!(old.as_str(), "running" | "failed" | "error")
                    {
                        let _ = app
                            .notification()
                            .builder()
                            .title(format!("✅ {} CI 通过", st.repo))
                            .body("全部检查通过 · all checks passed")
                            .show();
                    }
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
        eprintln!("[poller] cycle done in {:?}, sleeping {}s", cycle_start.elapsed(), secs);
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
fn get_config() -> Config {
    load_config()
}

#[tauri::command]
fn save_config(app: AppHandle, config: Config) {
    let _ = save_config_file(&config);
    wake(&app);
}

#[tauri::command]
fn refresh_now(app: AppHandle) {
    wake(&app);
}

#[tauri::command]
fn token_info_cmd() -> TokenInfo {
    let cfg = load_config();
    token_info(&cfg.token)
}

#[tauri::command]
fn import_local_dir() -> Vec<String> {
    scan_local_dir()
}

#[tauri::command]
fn save_token(app: AppHandle, token: String) -> Result<(), String> {
    let t = token.trim().to_string();
    if t.is_empty() {
        return Err("empty".into());
    }
    let entry = keyring_entry().ok_or("keyring unavailable")?;
    entry.set_password(&t).map_err(|e| e.to_string())?;
    wake(&app);
    Ok(())
}

#[tauri::command]
fn open_url(_app: AppHandle, url: String) {
    open_external(&url);
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

// ============ 入口 ============

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .on_window_event(|window, event| {
            if window.label() == "panel" {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let (tx, rx) = mpsc::channel::<()>();
            app.manage(Shared {
                snapshot: Mutex::new(Vec::new()),
                last_refresh_ms: Mutex::new(0),
                refreshing: AtomicBool::new(false),
                wake: Mutex::new(tx),
                device_flow_cancel: Mutex::new(None),
            });

            let menu = build_menu(app.handle(), &[], 0)?;
            TrayIconBuilder::with_id("main")
                .icon(tray_image(142, 142, 147))
                .tooltip("GitCIStatus")
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| handle_menu_event(app, event))
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("panel") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                if let Ok(ws) = w.outer_size() {
                                    // position 为托盘图标点击点，面板定位于其下方居中
                                    let x = position.x as i32 - (ws.width as i32) / 2;
                                    let y = position.y as i32 + 24;
                                    let _ = w.set_position(PhysicalPosition::new(x.max(0), y));
                                }
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            WebviewWindowBuilder::new(app, "panel", WebviewUrl::App("index.html".into()))
                .title("GitCIStatus")
                .inner_size(390.0, 560.0)
                .decorations(false)
                .resizable(false)
                .visible(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .build()?;

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
            import_local_dir,
            save_token,
            open_url,
            quit_app,
            log_js,
            device_flow_start,
            device_flow_abort
        ])
        .run(tauri::generate_context!())
        .expect("GitCIStatus 启动失败");
}
