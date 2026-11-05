// Glint 跨平台版前端：纯 JS，无构建步骤
// 通过 withGlobalTauri 暴露的 window.__TAURI__ 与 Rust 后端通信

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// JS 错误上报到后端日志，便于排查面板问题
function report(msg) {
  try { invoke('log_js', { msg: String(msg) }); } catch (e) { /* ignore */ }
}
window.addEventListener('error', (e) => report(`error: ${e.message} @ ${e.filename}:${e.lineno}`));
window.addEventListener('unhandledrejection', (e) => report(`rejection: ${e.reason}`));

// ============ i18n ============

const I18N = {
  zh: {
    pass: '通过', failed: '失败', running: '运行中', noChecks: '无 CI',
    error: '错误', loading: '加载中…', failedSuffix: '失败', failedN: '{n} 项失败',
    manage: '管理', refresh: '刷新', quit: '退出', back: '返回',
    loadingRepos: '正在加载仓库状态…', emptyRepos: '还没有监控的仓库', addRepo: '添加仓库',
    failedDetail: '失败详情 · {n} 项', viewLog: '查看日志', more: '还有 {n} 项…',
    copy: '复制', copied: '已复制', copyFailed: '复制失败',
    copyError: '复制错误', copiedSummary: '已复制摘要', logUnavailable: '（日志拉取失败，仅摘要）',
    checks: '检查 {n} 项', stars: '{n}',
    allPass: '全部检查通过',
    tokenPresent: '已登录 GitHub',
    tokenMissing: '未登录：仅公共仓库，限流 60 次/时',
    onboardTitle: '通过 GitHub 登录，开始监控',
    onboardDesc: '登录后勾选要监控的仓库（含私有仓库），享受更高限流。Token 只存本机钥匙串，无服务器、无遥测。',
    advancedClientId: '高级：自定义 OAuth App Client ID',
    relogin: '重新登录',
    manageTitle: '仓库管理', reposSection: '监控仓库',
    pickFromGithub: '从 GitHub 选择仓库',
    fetchRepos: '拉取我的仓库', fetching: '拉取中…', filterPh: '筛选…', listLoading: '正在拉取仓库列表…',
    checkHint: '勾选即加入监控，取消勾选即移除；可随时重新拉取。',
    noMatch: '没有匹配的仓库', emptyFetched: '未拉取到仓库', fetchFailed: '拉取失败：{msg}',
    refreshNotify: '刷新与通知', interval: '刷新间隔',
    notifyFailure: '失败时发系统通知', notifySuccess: '运行中转为通过时发通知',
    autostart: '开机自启（登录时自动启动）',
    autostartOn: '已开启开机自启（下次登录生效）', autostartOff: '已关闭开机自启',
    autostartFailed: '设置失败：{msg}',
    updateAvailable: '新版本 v{v} 可用', updateDownload: '下载',
    aboutSection: '关于', checkUpdate: '检查更新', upToDate: '已是最新版本',
    newAvailShort: '新版本 v{v} 可用', checkUpdateFailed: '检查失败：{msg}',
    rateLimitHint: 'GitHub API 限流中，登录后限额提升至 5000 次/时',
    rateLimitHintAuthed: 'GitHub API 限流中，稍后自动恢复',
    goLogin: '去登录',
    clientIdPh: 'OAuth App Client ID（留空使用内置默认）',
    deviceLogin: '通过 GitHub 登录', cancel: '取消',
    needClientId: '请先填写 OAuth Client ID：github.com → Settings → Developer settings → OAuth Apps 注册应用（无需 secret），把 Client ID 粘贴到这里',
    deviceWait: '已打开浏览器，请授权以下代码：', deviceFallback: '若未自动打开，访问：',
    deviceSuccess: 'GitHub 登录成功，Token 已存入钥匙串',
    deviceWaiting: '等待授权中…',
    deviceFailed: '登录失败：{msg}',
    deviceErrors: {
      cancelled: '已取消', expired: '授权码已过期，请重试', denied: '你拒绝了授权',
      keyring_failed: 'Token 获取成功但写入钥匙串失败', bad_response: '响应异常，请重试',
      NO_CLIENT_ID: '未配置 Client ID', HTTP_404: 'Client ID 无效，请检查是否复制完整',
      NO_TOKEN: '未登录且无 Token，请先通过 GitHub 登录',
    },
    justNow: '刚刚',
    err401: 'Token 无效 (401)', err403: '无权限或限流 (403)',
    err404: '仓库不存在或无权限 (404)', errNetwork: '网络错误：{msg}',
    statusText: {
      success: '通过', failed: '失败', running: '运行中',
      no_checks: '无 CI', error: '错误', loading: '加载中…',
    },
  },
  en: {
    pass: 'Pass', failed: 'Failed', running: 'Running', noChecks: 'No CI',
    error: 'Error', loading: 'Loading…', failedSuffix: 'failed', failedN: '{n} failed',
    manage: 'Manage', refresh: 'Refresh', quit: 'Quit', back: 'Back',
    loadingRepos: 'Loading repository status…', emptyRepos: 'No repositories yet', addRepo: 'Add Repository',
    failedDetail: 'Failures · {n}', viewLog: 'View log', more: '{n} more…',
    copy: 'Copy', copied: 'Copied', copyFailed: 'Copy failed',
    copyError: 'Copy error', copiedSummary: 'Copied summary', logUnavailable: '(log unavailable, summary only)',
    checks: '{n} checks', stars: '{n}',
    allPass: 'All checks passed',
    tokenPresent: 'Signed in with GitHub',
    tokenMissing: 'Not signed in: public repos only, 60 req/h',
    onboardTitle: 'Sign in with GitHub to start monitoring',
    onboardDesc: 'Pick repositories to watch (private ones included) with higher rate limits. The token stays in your system keychain — no server, no telemetry.',
    advancedClientId: 'Advanced: custom OAuth App Client ID',
    relogin: 'Sign in again',
    manageTitle: 'Manage Repositories', reposSection: 'Repositories',
    pickFromGithub: 'Select repos from GitHub',
    fetchRepos: 'Fetch my repos', fetching: 'Fetching…', filterPh: 'Filter…', listLoading: 'Loading repository list…',
    checkHint: 'Check to monitor, uncheck to remove; re-fetch anytime.',
    noMatch: 'No matching repos', emptyFetched: 'No repos found', fetchFailed: 'Fetch failed: {msg}',
    refreshNotify: 'Refresh & Notifications', interval: 'Interval',
    notifyFailure: 'Notify on failure', notifySuccess: 'Notify when running → pass',
    autostart: 'Launch at login',
    autostartOn: 'Launch at login enabled (applies at next sign-in)', autostartOff: 'Launch at login disabled',
    autostartFailed: 'Failed to set: {msg}',
    updateAvailable: 'New version v{v} available', updateDownload: 'Download',
    aboutSection: 'About', checkUpdate: 'Check for updates', upToDate: 'Up to date',
    newAvailShort: 'New version v{v} available', checkUpdateFailed: 'Check failed: {msg}',
    rateLimitHint: 'GitHub API rate limited — sign in for 5,000 req/h',
    rateLimitHintAuthed: 'GitHub API rate limited, will recover shortly',
    goLogin: 'Sign in',
    clientIdPh: 'OAuth App Client ID (leave empty to use built-in default)',
    deviceLogin: 'Sign in with GitHub', cancel: 'Cancel',
    needClientId: 'Fill in the OAuth Client ID first: github.com → Settings → Developer settings → OAuth Apps → New OAuth App (no secret needed), then paste the Client ID here',
    deviceWait: 'Browser opened, authorize this code:', deviceFallback: 'If not opened, visit:',
    deviceSuccess: 'Signed in with GitHub, token saved to keychain',
    deviceWaiting: 'Waiting for authorization…',
    deviceFailed: 'Sign-in failed: {msg}',
    deviceErrors: {
      cancelled: 'Cancelled', expired: 'Code expired, try again', denied: 'Authorization denied',
      keyring_failed: 'Token obtained but failed to save to keychain', bad_response: 'Unexpected response, try again',
      NO_CLIENT_ID: 'Client ID not configured', HTTP_404: 'Invalid Client ID, please check it',
      NO_TOKEN: 'No token: sign in with GitHub first',
    },
    justNow: 'just now',
    err401: 'Invalid token (401)', err403: 'Forbidden or rate limited (403)',
    err404: 'Repo not found or no access (404)', errNetwork: 'Network error: {msg}',
    statusText: {
      success: 'Pass', failed: 'Failed', running: 'Running',
      no_checks: 'No CI', error: 'Error', loading: 'Loading…',
    },
  },
};

let lang = ''; // "zh" | "en"，空 = 待检测

function detectLang() {
  const nav = (navigator.language || 'en').toLowerCase();
  return nav.startsWith('zh') ? 'zh' : 'en';
}

function resolveKey(dict, key) {
  // 支持 "statusText.success" 点分路径，也兼容扁平 key
  return key.split('.').reduce((o, k) => (o && o[k] !== undefined ? o[k] : undefined), dict);
}

function t(key, params) {
  const dict = I18N[lang] || I18N.zh;
  let text = resolveKey(dict, key);
  if (text === undefined) text = resolveKey(I18N.en, key);
  if (text === undefined) text = key;
  if (typeof text === 'string' && params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, v);
    }
  }
  return text;
}

function applyI18n() {
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  document.querySelectorAll('[data-i18n-ph]').forEach((el) => {
    el.placeholder = t(el.dataset.i18nPh);
  });
  document.getElementById('lang-btn').textContent = lang === 'zh' ? 'EN' : '中文';
  document.getElementById('onboard-lang-btn').textContent = lang === 'zh' ? 'EN' : '中文';
}

// ============ 时间显示 ============

function relativeTime(iso) {
  if (!iso) return '';
  const date = new Date(iso);
  const diff = Date.now() - date.getTime();
  if (Math.abs(diff) < 10000) return t('justNow');
  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto', style: 'narrow' });
  const abs = Math.abs(diff);
  if (abs < 60000) return rtf.format(-Math.round(diff / 1000), 'second');
  if (abs < 3600000) return rtf.format(-Math.round(diff / 60000), 'minute');
  if (abs < 86400000) return rtf.format(-Math.round(diff / 3600000), 'hour');
  return rtf.format(-Math.round(diff / 86400000), 'day');
}

function localizedError(text) {
  if (!text) return '';
  const m = text.match(/^HTTP_(\d+)$/);
  if (m) {
    const dict = I18N[lang] || I18N.zh;
    const key = 'err' + m[1];
    return dict[key] || text;
  }
  if (text.startsWith('NETWORK ')) return t('errNetwork', { msg: text.slice(8) });
  return text;
}

function durationText(startIso, endIso) {
  if (!startIso || !endIso) return '';
  const ms = new Date(endIso) - new Date(startIso);
  if (!(ms > 0)) return '';
  const total = Math.round(ms / 1000);
  if (total < 60) return `${total}s`;
  const m = Math.floor(total / 60);
  if (m < 60) return `${m}m${total % 60}s`;
  return `${Math.floor(m / 60)}h${m % 60}m`;
}

// ============ 状态映射 ============

const STATUS_COLOR = {
  success: 'var(--green)', failed: 'var(--red)', running: 'var(--orange)',
  no_checks: 'var(--gray)', error: 'var(--purple)', loading: 'var(--gray)',
};

function overallOf(repos) {
  let failed = 0, running = false, success = false, errored = false;
  for (const s of repos) {
    if (s.status === 'failed') failed += 1;
    else if (s.status === 'running') running = true;
    else if (s.status === 'success') success = true;
    else if (s.status === 'error') errored = true;
  }
  if (failed > 0) return { key: 'failed', failed };
  if (running) return { key: 'running', failed: 0 };
  if (errored && !success) return { key: 'error', failed: 0 };
  if (success) return { key: 'success', failed: 0 };
  return { key: 'no_checks', failed: 0 };
}

// ============ 渲染 ============

const expanded = new Set(); // 展开的仓库名
let snapshot = { repos: [], last_refresh_ms: 0, refreshing: false };

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

// ============ SVG 图标（Lucide 风格：24 viewBox / stroke 2 / currentColor） ============

const ICON_PATHS = {
  settings: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  'refresh-cw': '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
  power: '<path d="M12 2v10"/><path d="M18.4 6.6a9 9 0 1 1-12.77.04"/>',
  'chevron-right': '<path d="m9 18 6-6-6-6"/>',
  'chevron-left': '<path d="m15 18-6-6 6-6"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  'alert-triangle': '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  star: '<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>',
  lock: '<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
  'arrow-up-circle': '<circle cx="12" cy="12" r="10"/><path d="m16 12-4-4-4 4"/><path d="M12 16V8"/>',
  'external-link': '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h8"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  slash: '<circle cx="12" cy="12" r="10"/><line x1="9" y1="15" x2="15" y2="9"/>',
  'git-branch': '<line x1="6" x2="6" y1="3" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/>',
  'git-pr': '<circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" x2="6" y1="9" y2="21"/>',
  'circle-dot': '<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="1.2" fill="currentColor" stroke="none"/>',
  play: '<polygon points="6 3 20 12 6 21 6 3"/>',
};

function icon(name, size = 12) {
  const paths = ICON_PATHS[name] || '';
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
}

// 静态 HTML 里的 data-icon 占位在启动时注入一次
function injectStaticIcons() {
  document.querySelectorAll('[data-icon]').forEach((node) => {
    node.innerHTML = icon(node.dataset.icon);
  });
}

// ============ 剪贴板 ============

// 优先 Tauri 剪贴板插件（跨平台可靠），失败时退回浏览器 API，最后 execCommand 兜底
async function copyText(text) {
  try {
    await window.__TAURI__.clipboardManager.writeText(text);
    return true;
  } catch (e) { /* 继续尝试浏览器 API */ }
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (e2) { /* 继续兜底 */ }
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand('copy');
    ta.remove();
    if (!ok) report('execCommand copy returned false');
    return ok;
  } catch (e3) {
    report(`copy failed: ${e3}`);
    return false;
  }
}

// 点击后短暂显示复制结果，再恢复原文案
function flashCopyBtn(btn, ok, okLabel, restoreLabel) {
  btn.textContent = ok ? okLabel : t('copyFailed');
  btn.classList.toggle('fail', !ok);
  setTimeout(() => {
    btn.textContent = restoreLabel;
    btn.classList.remove('fail');
  }, 1600);
}

// 失败摘要（复制错误的前置头：仓库 @ 提交 / job ▸ step / 链接）
function failureSummaryText(s, entry) {
  const lines = [
    `${s.repo}${s.head_sha ? ` @ ${s.head_sha.slice(0, 7)}` : ''}${s.branch ? ` (${s.branch})` : ''}`,
  ];
  if (entry) {
    let job = `Job: ${entry.name}`;
    if (entry.step) job += ` ▸ ${entry.step}`;
    lines.push(job);
  }
  const url = (entry && entry.url) || s.top_fail_url;
  if (url) lines.push(url);
  return lines.join('\n');
}

function compactStatusText(s) {
  if (s.status === 'failed') {
    if (s.failed_count === 1 && s.failed_checks && s.failed_checks[0]) {
      return `${s.failed_checks[0].name} ${t('failedSuffix')}`;
    }
    return t('failedN', { n: s.failed_count });
  }
  return t(`statusText.${s.status}`) || s.status;
}

function renderRepoCard(container, s) {
  const item = el('div', 'repo-item');
  const row = el('div', 'repo-row');

  const dot = el('i', `dot${s.status === 'running' ? ' pulse' : ''}`);
  dot.style.background = STATUS_COLOR[s.status] || 'var(--gray)';
  row.appendChild(dot);

  row.appendChild(el('span', 'name', s.repo));
  row.appendChild(el('span', 'spacer', ''));

  const statusEl = el('span', 'status-text', compactStatusText(s));
  statusEl.style.color = STATUS_COLOR[s.status] || 'var(--text-2)';
  row.appendChild(statusEl);

  row.appendChild(el('span', 'time', relativeTime(s.rows?.[0]?.date || s.pushed_at)));

  const isExpanded = expanded.has(s.repo);
  const chevron = el('span', `chevron${isExpanded ? ' open' : ''}`);
  chevron.innerHTML = icon('chevron-right', 14);
  row.appendChild(chevron);

  row.addEventListener('click', () => {
    if (expanded.has(s.repo)) expanded.delete(s.repo);
    else expanded.add(s.repo);
    render();
  });
  item.appendChild(row);

  if (isExpanded) {
    item.appendChild(detailSection(s));
  }
  container.appendChild(item);
}

function detailSection(s) {
  const detail = el('div', 'detail');

  if (s.error_text) {
    detail.appendChild(el('div', 'error-text', localizedError(s.error_text)));
    return detail;
  }

  // 提交信息
  if (s.commit_subject || s.head_sha) {
    const line = el('div', 'commit-line');
    if (s.commit_subject) line.appendChild(el('span', 'subject', s.commit_subject));
    if (s.head_sha) line.appendChild(el('span', 'sha-chip', s.head_sha.slice(0, 7)));
    detail.appendChild(line);
  }

  // 检查统计
  if (s.total_checks > 0) {
    const stats = el('div', 'stats');
    stats.appendChild(el('span', 'stat-chip', t('checks', { n: s.total_checks })));
    if (s.failed_count > 0) {
      const bad = el('span', 'stat-chip bad');
      bad.innerHTML = `${icon('x', 10)}<span>${s.failed_count}</span>`;
      stats.appendChild(bad);
    }
    const good = el('span', 'stat-chip good');
    good.innerHTML = `${icon('check', 10)}<span>${s.passed_checks}</span>`;
    stats.appendChild(good);
    if (s.skipped_checks > 0) {
      const skip = el('span', 'stat-chip skip');
      skip.innerHTML = `${icon('slash', 10)}<span>${s.skipped_checks}</span>`;
      stats.appendChild(skip);
    }
    detail.appendChild(stats);
  }

  // 失败详情
  if (s.status === 'failed') {
    // id 仅 failed_jobs 有意义（Actions job id，日志端点用它）；failed_checks 是 check run id，不能混用
    const entries = (s.failed_jobs && s.failed_jobs.length)
      ? s.failed_jobs.map((j) => ({ id: j.id, name: j.name, step: j.failed_step, url: j.url, start: j.started_at, end: j.completed_at }))
      : (s.failed_checks || []).map((c) => ({ id: null, name: c.name, step: null, url: c.url, start: c.started_at, end: c.completed_at }));
    if (entries.length) {
      const box = el('div', 'fail-box');
      const head = el('div', 'fail-head');
      const headLabel = el('span', '', t('failedDetail', { n: s.failed_count }));
      head.innerHTML = icon('alert-triangle', 12);
      head.appendChild(headLabel);
      if (s.top_fail_url) {
        const log = el('a', '');
        log.innerHTML = `${t('viewLog')} ${icon('external-link', 11)}`;
        log.addEventListener('click', (e) => { e.stopPropagation(); invoke('open_url', { url: s.top_fail_url }); });
        head.appendChild(log);
      }
      // 核心承诺：失败后不跳网页，直接复制错误（日志摘录）
      const copyBtn = el('button', 'copy-btn', t('copyError'));
      copyBtn.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (copyBtn.disabled) return;
        copyBtn.disabled = true;
        const entry = entries[0];
        let text = failureSummaryText(s, entry);
        let gotExcerpt = false;
        if (entry && entry.id) {
          try {
            const excerpt = await invoke('fetch_job_log_excerpt', { repo: s.repo, jobId: entry.id });
            if (excerpt && excerpt.trim()) {
              text = `${text}\n\n${excerpt}`;
              gotExcerpt = true;
            }
          } catch (err) {
            report(`fetch_job_log_excerpt failed: ${err}`);
          }
        }
        if (!gotExcerpt) text = `${text}\n${t('logUnavailable')}`;
        const ok = await copyText(text);
        flashCopyBtn(copyBtn, ok, gotExcerpt ? t('copied') : t('copiedSummary'), t('copyError'));
        copyBtn.disabled = false;
      });
      head.appendChild(copyBtn);
      box.appendChild(head);
      for (const entry of entries.slice(0, 6)) {
        const item = el('div', 'fail-item');
        const x = el('span', 'x');
        x.innerHTML = icon('x', 11);
        item.appendChild(x);
        item.appendChild(el('span', 'name', entry.name));
        if (entry.step) item.appendChild(el('span', 'step', `› ${entry.step}`));
        item.appendChild(el('span', 'spacer', ''));
        const dur = el('span', 'dur', durationText(entry.start, entry.end));
        item.appendChild(dur);
        item.addEventListener('click', () => { if (entry.url) invoke('open_url', { url: entry.url }); });
        box.appendChild(item);
      }
      if (entries.length > 6) box.appendChild(el('div', 'muted small', t('more', { n: entries.length - 6 })));
      detail.appendChild(box);
    }
  }

  // 最近运行
  if (s.rows && s.rows.length) {
    const list = el('div', 'run-list');
    for (const r of s.rows.slice(0, 4)) {
      const item = el('div', 'run-item');
      const dot = el('i', 'dot');
      dot.style.background = STATUS_COLOR[r.state] || 'var(--purple)';
      item.appendChild(dot);
      item.appendChild(el('span', 'run-title', r.title));
      item.appendChild(el('span', 'run-time', relativeTime(r.date)));
      item.addEventListener('click', () => { if (r.url) invoke('open_url', { url: r.url }); });
      list.appendChild(item);
    }
    detail.appendChild(list);
  }

  // 开放 issues（后端按最近活跃排序，取前 8 条）
  if (Array.isArray(s.issues) && s.issues.length) {
    const box = el('div', 'issue-list');
    const head = el('div', 'issue-head');
    head.innerHTML = icon('circle-dot', 12);
    head.appendChild(el('span', '', t('issuesSection', { n: s.open_issues })));
    const all = el('a', '');
    all.innerHTML = `${t('viewAllIssues')} ${icon('external-link', 11)}`;
    all.addEventListener('click', (e) => { e.stopPropagation(); invoke('open_url', { url: `https://github.com/${s.repo}/issues` }); });
    head.appendChild(all);
    box.appendChild(head);
    for (const it of s.issues.slice(0, 8)) {
      const item = el('div', 'issue-item');
      const top = el('div', 'issue-title');
      top.appendChild(el('span', 'issue-num', `#${it.number}`));
      top.appendChild(el('span', 'issue-name', it.title));
      item.appendChild(top);
      const meta = el('div', 'issue-meta');
      for (const l of (it.labels || []).slice(0, 3)) {
        if (!l.name) continue;
        const tag = el('span', 'label-pill', l.name);
        if (l.color) {
          tag.style.background = `#${l.color}33`;
          tag.style.color = `#${l.color}`;
        }
        meta.appendChild(tag);
      }
      if (it.comments > 0) {
        const c = el('span', 'issue-comments');
        c.innerHTML = `${icon('message-square', 10)}<span>${it.comments}</span>`;
        meta.appendChild(c);
      }
      meta.appendChild(el('span', 'issue-time', relativeTime(it.updated_at || it.created_at)));
      item.appendChild(meta);
      item.addEventListener('click', () => { if (it.url) invoke('open_url', { url: it.url }); });
      box.appendChild(item);
    }
    if (s.issues.length > 8) box.appendChild(el('div', 'muted small', t('more', { n: s.issues.length - 8 })));
    detail.appendChild(box);
  } else if (Array.isArray(s.issues) && s.open_issues === 0) {
    const none = el('div', 'issue-none');
    none.innerHTML = `${icon('check', 11)}<span>${t('noOpenIssues')}</span>`;
    detail.appendChild(none);
  }

  // 链接
  if (s.open_prs >= 0) {
    const links = el('div', 'links');
    const mkLink = (ic, text, url) => {
      const a = el('a', 'link-chip');
      a.innerHTML = `${icon(ic, 11)}<span>${text}</span>`;
      a.addEventListener('click', () => invoke('open_url', { url }));
      return a;
    };
    links.appendChild(mkLink('git-pr', `PR ${s.open_prs}`, `https://github.com/${s.repo}/pulls`));
    links.appendChild(mkLink('circle-dot', `Issues ${s.open_issues}`, `https://github.com/${s.repo}/issues`));
    links.appendChild(mkLink('play', 'Actions', `https://github.com/${s.repo}/actions`));
    const stars = el('span', 'stars');
    stars.innerHTML = `${icon('star', 11)}<span>${s.stars}</span>`;
    links.appendChild(stars);
    detail.appendChild(links);
  }

  return detail;
}

function renderPanel() {
  const list = document.getElementById('repo-list');
  list.innerHTML = '';

  if (!snapshot.repos.length) {
    const empty = el('div', 'empty-state');
    const emptyIcon = el('div', 'empty-icon');
    emptyIcon.innerHTML = icon('git-branch', 36);
    empty.appendChild(emptyIcon);
    if (snapshot.refreshing) {
      empty.appendChild(el('div', 'spinner'));
      empty.appendChild(el('div', '', t('loadingRepos')));
    } else {
      empty.appendChild(el('div', '', t('emptyRepos')));
      const btn = el('button', '', t('addRepo'));
      btn.addEventListener('click', () => switchView('manage'));
      empty.appendChild(btn);
    }
    list.appendChild(empty);
  } else {
    const group = el('div', 'repo-group');
    for (const s of snapshot.repos) renderRepoCard(group, s);
    list.appendChild(group);
  }

  // 头部
  const overall = overallOf(snapshot.repos);
  const pill = document.getElementById('overall-pill');
  pill.className = `pill ${overall.key === 'failed' ? 'red' : overall.key === 'running' ? 'orange' : overall.key === 'success' ? 'green' : overall.key === 'error' ? 'purple' : 'gray'}`;
  const dotEl = pill.querySelector('.dot');
  dotEl.style.background = STATUS_COLOR[overall.key] || 'var(--gray)';
  document.getElementById('overall-text').textContent = t(`statusText.${overall.key}`);

  document.getElementById('spinner').classList.toggle('hidden', !snapshot.refreshing);
  document.getElementById('last-refresh').textContent = relativeTime(
    snapshot.last_refresh_ms ? new Date(snapshot.last_refresh_ms).toISOString() : null
  );
  document.getElementById('refresh-btn').disabled = snapshot.refreshing;
}

function renderManageRepos(config) {
  const list = document.getElementById('repo-manage-list');
  list.innerHTML = '';
  if (!config.repos.length) {
    list.appendChild(el('div', 'muted small', t('emptyRepos')));
  }
  for (const repo of config.repos) {
    const item = el('div', 'repo-manage-item');
    item.appendChild(el('span', 'name', repo));
    const rm = el('button', 'remove');
    rm.innerHTML = icon('x', 12);
    rm.addEventListener('click', async () => {
      config.repos = config.repos.filter((r) => r !== repo);
      await invoke('save_config', { config });
      renderManageRepos(config);
      renderTokenInfo();
    });
    item.appendChild(rm);
    list.appendChild(item);
  }
}

let currentConfig = null;

async function loadManage() {
  currentConfig = await invoke('get_config');
  document.getElementById('interval-select').value = String(currentConfig.interval_seconds);
  document.getElementById('notify-failure').checked = currentConfig.notify_on_failure;
  document.getElementById('notify-success').checked = currentConfig.notify_on_success;
  document.getElementById('autostart').checked = await invoke('autostart_state');
  document.getElementById('client-id-input').value = currentConfig.oauth_client_id || '';
  document.getElementById('about-version').textContent = 'Glint v' + (await invoke('app_version'));
  renderManageRepos(currentConfig);
  renderCheckList(); // 保留上次拉取的仓库列表
}

async function persistConfig() {
  if (!currentConfig) return;
  currentConfig.interval_seconds = Number(document.getElementById('interval-select').value);
  currentConfig.notify_on_failure = document.getElementById('notify-failure').checked;
  currentConfig.notify_on_success = document.getElementById('notify-success').checked;
  currentConfig.oauth_client_id = document.getElementById('client-id-input').value.trim();
  await invoke('save_config', { config: currentConfig });
}

async function showManageMessage(text) {
  const box = document.getElementById('manage-message');
  box.textContent = text;
  box.classList.remove('hidden');
  clearTimeout(showManageMessage._timer);
  showManageMessage._timer = setTimeout(() => box.classList.add('hidden'), 4000);
}

// 登录态决定视图：未登录全屏引导，登录成功自动进入面板首页；
// 「重新登录」走 applyAuthState(false) 回引导页，取消授权后凭旧会话返回首页
function applyAuthState(present) {
  const was = authed;
  authed = present;
  if (!present) {
    switchView('onboarding');
  } else if (!was || currentView === 'onboarding') {
    switchView('panel');
  }
}

async function renderTokenInfo() {
  const info = await invoke('token_info_cmd');
  report(`token_info: present=${info.present} source=${info.source}`);
  const text = info.present ? t('tokenPresent') : t('tokenMissing');
  document.getElementById('token-info').textContent = text;
  document.getElementById('token-manage-info').textContent = text;
  applyAuthState(info.present);
}

let currentView = 'panel';
let authed = false;

function switchView(view) {
  // 未登录时强制停留在登录引导页
  if (view !== 'onboarding' && !authed) view = 'onboarding';
  currentView = view;
  document.getElementById('panel-view').classList.toggle('hidden', view !== 'panel');
  document.getElementById('manage-view').classList.toggle('hidden', view !== 'manage');
  document.getElementById('onboarding-view').classList.toggle('hidden', view !== 'onboarding');
  if (view === 'manage') loadManage();
}

// 限流提示：任一仓库处于 403 错误时在面板顶部出横幅（未登录附「去登录」入口）
function renderRateHint() {
  const any403 = (snapshot?.repos || []).some(
    (s) => s.status === 'error' && (s.error_text || '').includes('HTTP_403')
  );
  const banner = document.getElementById('rate-banner');
  if (!any403) {
    banner.classList.add('hidden');
    return;
  }
  document.getElementById('rate-text').textContent = t(authed ? 'rateLimitHintAuthed' : 'rateLimitHint');
  const btn = document.getElementById('rate-btn');
  btn.classList.toggle('hidden', authed);
  btn.textContent = t('goLogin');
  btn.onclick = () => switchView('manage');
  banner.classList.remove('hidden');
}

function render() {
  try {
    renderPanel();
    renderRateHint();
    applyI18n();
  } catch (e) {
    report(`render failed: ${e && e.stack ? e.stack : e}`);
  }
}

// ============ 事件绑定 ============

function localizedDeviceError(key) {
  if (!key) return '';
  const dict = I18N[lang] || I18N.zh;
  return (dict.deviceErrors && dict.deviceErrors[key]) || key;
}

function setDeviceStatus(node) {
  const box = document.getElementById('device-flow-status');
  box.innerHTML = '';
  if (!node) {
    box.classList.add('hidden');
    return;
  }
  box.appendChild(node);
  box.classList.remove('hidden');
}

function deviceWaitingNode(info) {
  const wrap = el('div', '');
  wrap.appendChild(el('div', '', t('deviceWait')));
  const codeRow = el('div', 'user-code-row');
  codeRow.appendChild(el('div', 'user-code', info.user_code));
  const copyBtn = el('button', 'copy-btn', t('copy'));
  copyBtn.addEventListener('click', async (e) => {
    e.stopPropagation();
    if (copyBtn.disabled) return;
    copyBtn.disabled = true;
    const ok = await copyText(info.user_code);
    flashCopyBtn(copyBtn, ok, t('copied'), t('copy'));
    copyBtn.disabled = false;
  });
  codeRow.appendChild(copyBtn);
  wrap.appendChild(codeRow);
  const row = el('div', '');
  row.appendChild(el('span', 'muted small', `${t('deviceWaiting')} · `));
  const a = el('a', '', info.verification_url);
  a.addEventListener('click', () => invoke('open_url', { url: info.verification_url }));
  row.appendChild(a);
  wrap.appendChild(row);
  return wrap;
}

let fetchedRepos = []; // 最近一次拉取的可监控仓库
let repoFilter = '';
let fetchingRepos = false;

async function fetchUserRepos() {
  const btn = document.getElementById('fetch-repos-btn');
  if (fetchingRepos) return;
  btn.disabled = true;
  // 按钮内含图标 + 文字两个 span，只能改文字那个，不能动 textContent（会抹掉图标）
  const label = btn.querySelector('span[data-i18n]') || btn;
  const oldText = label.textContent;
  label.textContent = t('fetching');
  btn.classList.add('busy');
  fetchingRepos = true;
  renderCheckList(); // 立即出现加载态，不等网络返回
  try {
    fetchedRepos = await invoke('fetch_user_repos');
    if (!fetchedRepos.length) showManageMessage(t('emptyFetched'));
  } catch (e) {
    showManageMessage(t('fetchFailed', { msg: localizedDeviceError(String(e)) }));
  } finally {
    fetchingRepos = false;
    btn.disabled = false;
    btn.classList.remove('busy');
    label.textContent = oldText;
    renderCheckList();
  }
}

function renderCheckList() {
  const box = document.getElementById('repo-check-list');
  const filterInput = document.getElementById('repo-filter-input');
  box.innerHTML = '';
  if (!fetchedRepos.length) {
    // 首次拉取中：列表区显示 spinner，而不是一片空白
    if (fetchingRepos) {
      box.classList.remove('hidden');
      filterInput.classList.add('hidden');
      const loading = el('div', 'repo-check-loading');
      loading.appendChild(el('div', 'spinner'));
      loading.appendChild(el('span', 'muted small', t('listLoading')));
      box.appendChild(loading);
    } else {
      box.classList.add('hidden');
      filterInput.classList.add('hidden');
    }
    return;
  }
  box.classList.remove('hidden');
  filterInput.classList.remove('hidden');
  const kw = repoFilter.trim().toLowerCase();
  let shown = 0;
  for (const r of fetchedRepos) {
    if (kw && !r.full_name.toLowerCase().includes(kw) && !(r.description || '').toLowerCase().includes(kw)) continue;
    shown += 1;
    const item = el('div', 'repo-check-item');
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.checked = !!(currentConfig && currentConfig.repos.includes(r.full_name));
    cb.addEventListener('change', async () => {
      if (!currentConfig) return;
      if (cb.checked) {
        if (!currentConfig.repos.includes(r.full_name)) currentConfig.repos.push(r.full_name);
      } else {
        currentConfig.repos = currentConfig.repos.filter((x) => x !== r.full_name);
      }
      await invoke('save_config', { config: currentConfig });
      renderManageRepos(currentConfig);
    });
    item.appendChild(cb);
    item.appendChild(el('span', 'name', r.full_name));
    if (r.private) {
      const lock = el('span', 'lock');
      lock.innerHTML = icon('lock', 11);
      item.appendChild(lock);
    }
    if (r.description) item.appendChild(el('span', 'desc', r.description));
    item.addEventListener('click', (ev) => {
      if (ev.target !== cb) {
        cb.checked = !cb.checked;
        cb.dispatchEvent(new Event('change'));
      }
    });
    box.appendChild(item);
  }
  if (!shown) box.appendChild(el('div', 'muted small', t('noMatch')));
}

function bindEvents() {
  document.getElementById('manage-btn').addEventListener('click', () => switchView('manage'));
  document.getElementById('back-btn').addEventListener('click', () => switchView('panel'));

  document.getElementById('refresh-btn').addEventListener('click', () => invoke('refresh_now'));
  document.getElementById('quit-btn').addEventListener('click', () => invoke('quit_app'));

  const toggleLang = async () => {
    lang = lang === 'zh' ? 'en' : 'zh';
    if (currentConfig) {
      currentConfig.lang = lang;
      await invoke('save_config', { config: currentConfig });
    }
    render();
  };
  document.getElementById('lang-btn').addEventListener('click', toggleLang);
  document.getElementById('onboard-lang-btn').addEventListener('click', toggleLang);

  document.getElementById('fetch-repos-btn').addEventListener('click', fetchUserRepos);
  document.getElementById('repo-filter-input').addEventListener('input', (e) => {
    repoFilter = e.target.value;
    renderCheckList();
  });

  document.getElementById('interval-select').addEventListener('change', persistConfig);
  document.getElementById('notify-failure').addEventListener('change', persistConfig);
  document.getElementById('notify-success').addEventListener('change', persistConfig);

  document.getElementById('autostart').addEventListener('change', async (e) => {
    try {
      await invoke('set_autostart_cmd', { enable: e.target.checked });
      showManageMessage(t(e.target.checked ? 'autostartOn' : 'autostartOff'));
    } catch (err) {
      e.target.checked = !e.target.checked;
      showManageMessage(t('autostartFailed', { msg: String(err) }));
    }
  });

  document.getElementById('client-id-input').addEventListener('change', persistConfig);

  // 重新登录：回到登录引导态（复用 onboarding 的登录按钮与状态展示）
  document.getElementById('relogin-btn').addEventListener('click', () => applyAuthState(false));

  // 检查更新：结果直接展示在「关于」区内，有新版附带下载按钮
  document.getElementById('check-update-btn').addEventListener('click', async () => {
    const btn = document.getElementById('check-update-btn');
    const box = document.getElementById('about-status');
    btn.disabled = true;
    box.classList.remove('hidden');
    box.textContent = '…';
    try {
      const info = await invoke('check_update');
      box.textContent = '';
      if (info && info.has_update) {
        box.appendChild(document.createTextNode(t('newAvailShort', { v: info.latest }) + ' '));
        const dl = el('button', 'ghost-btn', t('updateDownload'));
        dl.addEventListener('click', () => invoke('open_url', { url: info.url }));
        box.appendChild(dl);
      } else {
        box.textContent = t('upToDate');
      }
    } catch (e) {
      box.textContent = t('checkUpdateFailed', { msg: String(e) });
    }
    btn.disabled = false;
  });

  const deviceLoginBtn = document.getElementById('device-login-btn');
  const deviceCancelBtn = document.getElementById('device-cancel-btn');
  let ignoreCancelUntil = 0; // 新会话启动后短暂忽略旧会话的 cancelled 事件

  deviceLoginBtn.addEventListener('click', async () => {
    // Client ID 留空时后端使用内置默认值
    const clientId = document.getElementById('client-id-input').value.trim();
    ignoreCancelUntil = Date.now() + 3000;
    deviceLoginBtn.disabled = true;
    deviceCancelBtn.classList.remove('hidden');
    try {
      const info = await invoke('device_flow_start', { clientId });
      setDeviceStatus(deviceWaitingNode(info));
    } catch (e) {
      deviceLoginBtn.disabled = false;
      deviceCancelBtn.classList.add('hidden');
      // 失败必须显示在当前视图内：manage-message 在管理视图，引导页看不到
      setDeviceStatus(el('div', 'device-error', t('deviceFailed', { msg: localizedDeviceError(String(e)) })));
    }
  });

  deviceCancelBtn.addEventListener('click', async () => {
    await invoke('device_flow_abort');
    // 状态清理由 device-flow 事件（cancelled）触发
  });

  listen('device-flow', async (event) => {
    const { ok, error } = event.payload || {};
    report(`device-flow event received: ok=${ok} error=${error || 'none'}`);
    if (!ok && error === 'cancelled' && Date.now() < ignoreCancelUntil) return;
    deviceLoginBtn.disabled = false;
    deviceCancelBtn.classList.add('hidden');
    if (ok) {
      setDeviceStatus(null);
      renderTokenInfo(); // 登录成功：applyAuthState 自动翻到面板首页
      fetchUserRepos();  // 并直接拉取可监控的仓库列表
    } else if (error !== 'cancelled') {
      // 失败必须显示在引导视图内，否则用户授权后卡在引导页无任何反馈
      setDeviceStatus(el('div', 'device-error', t('deviceFailed', { msg: localizedDeviceError(error) })));
    } else {
      setDeviceStatus(null);
    }
  });

  listen('status-updated', (event) => {
    report('event status-updated received');
    snapshot = event.payload;
    render();
  });

  listen('show-manage', () => switchView('manage'));

  // 兜底轮询：面板可见时每 5 秒拉一次快照，防止事件丢失导致界面停留在旧状态
  setInterval(async () => {
    if (document.hidden) return;
    try {
      const fresh = await invoke('get_snapshot');
      const stale = JSON.stringify(fresh) !== JSON.stringify(snapshot);
      if (stale) {
        report('poll picked up newer snapshot');
        snapshot = fresh;
        render();
      }
    } catch (e) {
      report(`poll failed: ${e}`);
    }
  }, 5000);
}

// ============ 检查更新（静默，失败不影响主功能） ============

async function checkUpdate() {
  try {
    const info = await invoke('check_update');
    if (!info || !info.has_update) return;
    document.getElementById('update-text').textContent = t('updateAvailable', { v: info.latest });
    const btn = document.getElementById('update-btn');
    btn.textContent = t('updateDownload');
    btn.onclick = () => invoke('open_url', { url: info.url });
    document.getElementById('update-banner').classList.remove('hidden');
  } catch (e) {
    report(`check update failed: ${e}`);
  }
}

// ============ 启动 ============

async function init() {
  try {
    report('init start');
    const cfg = await invoke('get_config');
    lang = cfg.lang || detectLang();
    currentConfig = cfg;

    snapshot = await invoke('get_snapshot');
    report(`init snapshot: ${snapshot.repos.length} repos, refreshing=${snapshot.refreshing}`);
    injectStaticIcons();
    bindEvents();
    render();
    renderTokenInfo();
    checkUpdate();
  } catch (e) {
    report(`init failed: ${e}`);
  }
}

init();
