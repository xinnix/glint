// GitCIStatus 跨平台版前端：纯 JS，无构建步骤
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
    failedDetail: '失败详情 · {n} 项', viewLog: '查看日志 ↗', more: '还有 {n} 项…',
    checks: '检查 {n} 项', stars: '★ {n}',
    allPass: '全部检查通过',
    tokenPresent: 'Token ✓（通过 GitHub 登录）',
    tokenMissing: '未登录：仅公共仓库，限流 60 次/时',
    onboardTitle: '通过 GitHub 登录，开始监控',
    onboardDesc: '登录后勾选要监控的仓库（含私有仓库），享受更高限流。Token 只存本机钥匙串，无服务器、无遥测。',
    advancedClientId: '高级：自定义 OAuth App Client ID',
    relogin: '重新登录',
    manageTitle: '仓库管理', reposSection: '监控仓库',
    repoInputPh: 'owner/repo，支持批量粘贴（空格/逗号/换行分隔）',
    add: '添加', importGh: '导入 gh 账号的仓库', importLocal: '扫描本地 code 目录',
    addedN: '已添加 {n} 个仓库', addedNone: '没有新增（每项格式应为 owner/repo）',
    fetchRepos: '⟳ 拉取我的仓库', fetching: '拉取中…', filterPh: '筛选…',
    checkHint: '勾选即加入监控，取消勾选即移除；可随时重新拉取。',
    noMatch: '没有匹配的仓库', emptyFetched: '未拉取到仓库', fetchFailed: '拉取失败：{msg}',
    refreshNotify: '刷新与通知', interval: '刷新间隔',
    notifyFailure: '失败时发系统通知', notifySuccess: '运行中转为通过时发通知',
    autostart: '开机自启（登录时自动启动）',
    autostartOn: '已开启开机自启（下次登录生效）', autostartOff: '已关闭开机自启',
    autostartFailed: '设置失败：{msg}',
    updateAvailable: '⬆ 新版本 v{v} 可用', updateDownload: '下载',
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
    failedDetail: 'Failures · {n}', viewLog: 'View log ↗', more: '{n} more…',
    checks: '{n} checks', stars: '★ {n}',
    allPass: 'All checks passed',
    tokenPresent: 'Token ✓ (signed in with GitHub)',
    tokenMissing: 'Not signed in: public repos only, 60 req/h',
    onboardTitle: 'Sign in with GitHub to start monitoring',
    onboardDesc: 'Pick repositories to watch (private ones included) with higher rate limits. The token stays in your system keychain — no server, no telemetry.',
    advancedClientId: 'Advanced: custom OAuth App Client ID',
    relogin: 'Sign in again',
    manageTitle: 'Manage Repositories', reposSection: 'Repositories',
    repoInputPh: 'owner/repo, bulk paste supported (space/comma/newline)',
    add: 'Add', importGh: "Import from gh account", importLocal: 'Scan local code dirs',
    addedN: 'Added {n} repos', addedNone: 'Nothing added (expected owner/repo)',
    fetchRepos: '⟳ Fetch my repos', fetching: 'Fetching…', filterPh: 'Filter…',
    checkHint: 'Check to monitor, uncheck to remove; re-fetch anytime.',
    noMatch: 'No matching repos', emptyFetched: 'No repos found', fetchFailed: 'Fetch failed: {msg}',
    refreshNotify: 'Refresh & Notifications', interval: 'Interval',
    notifyFailure: 'Notify on failure', notifySuccess: 'Notify when running → pass',
    autostart: 'Launch at login',
    autostartOn: 'Launch at login enabled (applies at next sign-in)', autostartOff: 'Launch at login disabled',
    autostartFailed: 'Failed to set: {msg}',
    updateAvailable: '⬆ New version v{v} available', updateDownload: 'Download',
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
  const card = el('div', 'repo-card');
  const row = el('div', 'repo-row');

  const dot = el('i', 'dot');
  dot.style.background = STATUS_COLOR[s.status] || 'var(--gray)';
  row.appendChild(dot);

  row.appendChild(el('span', 'name', s.repo));
  row.appendChild(el('span', 'spacer', ''));

  const statusEl = el('span', 'status-text', compactStatusText(s));
  statusEl.style.color = STATUS_COLOR[s.status] || 'var(--text-2)';
  row.appendChild(statusEl);

  row.appendChild(el('span', 'time', relativeTime(s.rows?.[0]?.date || s.pushed_at)));

  const isExpanded = expanded.has(s.repo);
  const chevron = el('span', `chevron${isExpanded ? ' open' : ''}`, '▶');
  row.appendChild(chevron);

  row.addEventListener('click', () => {
    if (expanded.has(s.repo)) expanded.delete(s.repo);
    else expanded.add(s.repo);
    render();
  });
  card.appendChild(row);

  if (isExpanded) {
    card.appendChild(detailSection(s));
  }
  container.appendChild(card);
}

function detailSection(s) {
  const detail = el('div', 'detail');

  if (s.error_text) {
    detail.appendChild(el('div', 'error-text', localizedError(s.error_text)));
    return detail;
  }

  // 提交信息
  if (s.commit_subject || s.head_sha) {
    const line = el('div', 'muted-line');
    if (s.commit_subject) {
      const subject = el('span', '', s.commit_subject);
      subject.style.whiteSpace = 'nowrap';
      subject.style.overflow = 'hidden';
      subject.style.textOverflow = 'ellipsis';
      line.appendChild(subject);
    }
    if (s.head_sha) line.appendChild(el('span', 'sha', s.head_sha.slice(0, 7)));
    detail.appendChild(line);
  }

  // 检查统计
  if (s.total_checks > 0) {
    const stats = el('div', 'stats');
    stats.appendChild(el('span', '', t('checks', { n: s.total_checks })));
    if (s.failed_count > 0) stats.appendChild(el('span', 'bad', `✗ ${s.failed_count}`));
    stats.appendChild(el('span', 'good', `✓ ${s.passed_checks}`));
    if (s.skipped_checks > 0) stats.appendChild(el('span', '', `⊘ ${s.skipped_checks}`));
    detail.appendChild(stats);
  }

  // 失败详情
  if (s.status === 'failed') {
    const entries = (s.failed_jobs && s.failed_jobs.length)
      ? s.failed_jobs.map((j) => ({ name: j.name, step: j.failed_step, url: j.url, start: j.started_at, end: j.completed_at }))
      : (s.failed_checks || []).map((c) => ({ name: c.name, step: null, url: c.url, start: c.started_at, end: c.completed_at }));
    if (entries.length) {
      const box = el('div', 'fail-box');
      const head = el('div', 'fail-head');
      head.appendChild(el('span', '', `⚠ ${t('failedDetail', { n: s.failed_count })}`));
      if (s.top_fail_url) {
        const log = el('a', '', t('viewLog'));
        log.addEventListener('click', (e) => { e.stopPropagation(); invoke('open_url', { url: s.top_fail_url }); });
        head.appendChild(log);
      }
      box.appendChild(head);
      for (const entry of entries.slice(0, 6)) {
        const item = el('div', 'fail-item');
        item.appendChild(el('span', 'x', '✕'));
        item.appendChild(el('span', '', entry.name));
        if (entry.step) item.appendChild(el('span', 'step', `▸ ${entry.step}`));
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

  // 链接
  if (s.open_prs >= 0) {
    const links = el('div', 'links');
    const mkLink = (text, url) => {
      const a = el('a', '', text);
      a.addEventListener('click', () => invoke('open_url', { url }));
      return a;
    };
    links.appendChild(mkLink(`PR ${s.open_prs}`, `https://github.com/${s.repo}/pulls`));
    links.appendChild(mkLink(`Issues ${s.open_issues}`, `https://github.com/${s.repo}/issues`));
    links.appendChild(mkLink('Actions ↗', `https://github.com/${s.repo}/actions`));
    links.appendChild(el('span', 'stars', t('stars', { n: s.stars })));
    detail.appendChild(links);
  }

  return detail;
}

function renderPanel() {
  const list = document.getElementById('repo-list');
  list.innerHTML = '';

  if (!snapshot.repos.length) {
    const empty = el('div', 'empty-state');
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
    for (const s of snapshot.repos) renderRepoCard(list, s);
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
    item.appendChild(el('span', '', repo));
    const rm = el('button', 'remove', '✕');
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

function render() {
  try {
    renderPanel();
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
  wrap.appendChild(el('div', 'user-code', info.user_code));
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

async function fetchUserRepos() {
  const btn = document.getElementById('fetch-repos-btn');
  btn.disabled = true;
  const oldText = btn.textContent;
  btn.textContent = t('fetching');
  try {
    fetchedRepos = await invoke('fetch_user_repos');
    if (!fetchedRepos.length) showManageMessage(t('emptyFetched'));
  } catch (e) {
    showManageMessage(t('fetchFailed', { msg: localizedDeviceError(String(e)) }));
  } finally {
    btn.disabled = false;
    btn.textContent = oldText;
    renderCheckList();
  }
}

function renderCheckList() {
  const box = document.getElementById('repo-check-list');
  const filterInput = document.getElementById('repo-filter-input');
  box.innerHTML = '';
  if (!fetchedRepos.length) {
    box.classList.add('hidden');
    filterInput.classList.add('hidden');
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
    item.appendChild(el('span', '', r.full_name));
    if (r.private) item.appendChild(el('span', 'lock', '🔒'));
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

  document.getElementById('add-repo-btn').addEventListener('click', async () => {
    const input = document.getElementById('new-repo-input');
    const raw = input.value.trim();
    if (!raw || !currentConfig) return;
    const candidates = raw.split(/[\s,]+/).filter((x) => /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(x));
    let added = 0;
    for (const repo of candidates) {
      if (!currentConfig.repos.includes(repo)) {
        currentConfig.repos.push(repo);
        added += 1;
      }
    }
    if (added > 0) {
      await invoke('save_config', { config: currentConfig });
      renderManageRepos(currentConfig);
      showManageMessage(t('addedN', { n: added }));
    } else {
      showManageMessage(t('addedNone'));
    }
    input.value = '';
  });

  document.getElementById('fetch-repos-btn').addEventListener('click', fetchUserRepos);
  document.getElementById('repo-filter-input').addEventListener('input', (e) => {
    repoFilter = e.target.value;
    renderCheckList();
  });
  document.getElementById('import-local-btn').addEventListener('click', async () => {
    if (!currentConfig) return;
    const repos = await invoke('import_local_dir');
    let added = 0;
    for (const repo of repos) {
      if (!currentConfig.repos.includes(repo)) {
        currentConfig.repos.push(repo);
        added += 1;
      }
    }
    if (added > 0) {
      await invoke('save_config', { config: currentConfig });
      renderManageRepos(currentConfig);
      renderCheckList();
    }
    showManageMessage(added > 0 ? t('addedN', { n: added }) : t('addedNone'));
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
      showManageMessage(t('deviceFailed', { msg: localizedDeviceError(String(e)) }));
    }
  });

  deviceCancelBtn.addEventListener('click', async () => {
    await invoke('device_flow_abort');
    // 状态清理由 device-flow 事件（cancelled）触发
  });

  listen('device-flow', async (event) => {
    const { ok, error } = event.payload || {};
    if (!ok && error === 'cancelled' && Date.now() < ignoreCancelUntil) return;
    deviceLoginBtn.disabled = false;
    deviceCancelBtn.classList.add('hidden');
    setDeviceStatus(null);
    if (ok) {
      showManageMessage(t('deviceSuccess'));
      renderTokenInfo();
      fetchUserRepos(); // 登录成功后直接展示可监控的仓库列表
    } else if (error !== 'cancelled') {
      showManageMessage(t('deviceFailed', { msg: localizedDeviceError(error) }));
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
    bindEvents();
    render();
    renderTokenInfo();
    checkUpdate();
  } catch (e) {
    report(`init failed: ${e}`);
  }
}

init();
