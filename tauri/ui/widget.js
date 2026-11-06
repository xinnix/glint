// Glint 桌面组件：贴桌面层的紧凑状态列表。
// 数据与主面板共享同一 Rust 轮询管线（status-updated 事件广播到所有 webview），
// 本文件刻意不依赖 app.js（那是 panel 页的完整应用），只保留 widget 需要的最小子集。

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

// JS 错误上报到后端日志，便于排查组件问题
function report(msg) {
  try { invoke('log_js', { msg: String(msg) }); } catch (e) { /* ignore */ }
}
window.addEventListener('error', (e) => report(`widget error: ${e.message}`));
window.addEventListener('unhandledrejection', (e) => report(`widget rejection: ${e.reason}`));

// ============ i18n（app.js 字典的 widget 子集，双语硬编码与 Rust 菜单同风格） ============

const I18N = {
  zh: {
    refresh: '刷新', openPanel: '打开主面板', closeWidget: '关闭桌面组件',
    emptyRepos: '还没有监控的仓库', addRepo: '添加仓库', noChecks: '无 CI',
    dragHint: '按住顶部拖到喜欢的位置',
    pin: '固定位置', unpin: '取消固定',
    statusText: {
      success: '通过', failed: '失败', running: '运行中',
      no_checks: '无 CI', error: '错误', loading: '加载中…',
    },
  },
  en: {
    refresh: 'Refresh', openPanel: 'Open Panel', closeWidget: 'Close Widget',
    emptyRepos: 'No repositories yet', addRepo: 'Add Repository', noChecks: 'No CI',
    dragHint: 'Drag the header to place it anywhere',
    pin: 'Pin position', unpin: 'Unpin',
    statusText: {
      success: 'Pass', failed: 'Failed', running: 'Running',
      no_checks: 'No CI', error: 'Error', loading: 'Loading…',
    },
  },
};

let lang = '';

function detectLang() {
  const nav = (navigator.language || 'en').toLowerCase();
  return nav.startsWith('zh') ? 'zh' : 'en';
}

function t(key) {
  const dict = I18N[lang] || I18N.zh;
  return dict[key] !== undefined ? dict[key] : (I18N.en[key] !== undefined ? I18N.en[key] : key);
}

function tStatus(key) {
  const d = (I18N[lang] || I18N.zh).statusText;
  const e = I18N.en.statusText;
  return d[key] !== undefined ? d[key] : (e[key] !== undefined ? e[key] : key);
}

function applyI18n() {
  document.getElementById('w-ctx-refresh').textContent = t('refresh');
  document.getElementById('w-ctx-panel').textContent = t('openPanel');
  document.getElementById('w-ctx-close').textContent = t('closeWidget');
  document.getElementById('w-hint').textContent = t('dragHint');
  applyPinUI();
}

// ============ 状态映射与工具（与 app.js 保持一致） ============

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
  return { key: repos.length ? 'loading' : 'no_checks', failed: 0 };
}

function relativeTime(iso) {
  if (!iso) return '';
  const date = new Date(iso);
  const diff = Date.now() - date.getTime();
  if (Math.abs(diff) < 10000) return lang === 'zh' ? '刚刚' : 'now';
  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto', style: 'narrow' });
  const abs = Math.abs(diff);
  if (abs < 60000) return rtf.format(-Math.round(diff / 1000), 'second');
  if (abs < 3600000) return rtf.format(-Math.round(diff / 60000), 'minute');
  if (abs < 86400000) return rtf.format(-Math.round(diff / 3600000), 'hour');
  return rtf.format(-Math.round(diff / 86400000), 'day');
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

// widget 页只用到的图标（refresh-cw 与 app.js 同款；more-h/pin 是 Lucide 同名图标）
const ICON_PATHS = {
  'refresh-cw': '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
  'more-h': '<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>',
  pin: '<path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1z"/>',
};

function icon(name, size = 13) {
  const paths = ICON_PATHS[name] || '';
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
}

// ============ 渲染 ============

let snapshot = { repos: [], last_refresh_ms: 0, refreshing: false };

function renderWidget() {
  const list = document.getElementById('w-list');
  list.innerHTML = '';

  if (!snapshot.repos.length) {
    const empty = el('div', 'w-empty');
    if (snapshot.refreshing) {
      empty.appendChild(el('div', 'spinner'));
      empty.appendChild(el('div', 'muted', lang === 'zh' ? '加载中…' : 'Loading…'));
    } else {
      empty.appendChild(el('div', 'muted', t('emptyRepos')));
      const btn = el('button', '', t('addRepo'));
      btn.addEventListener('click', () => invoke('show_panel_cmd'));
      empty.appendChild(btn);
    }
    list.appendChild(empty);
  } else {
    const group = el('div', 'w-group');
    // snapshot.repos 由 Rust 侧按严重度排序（失败置顶），直接渲染
    for (const s of snapshot.repos) {
      const row = el('div', 'w-row');

      const dot = el('i', `dot${s.status === 'running' ? ' pulse' : ''}`);
      dot.style.background = STATUS_COLOR[s.status] || 'var(--gray)';
      row.appendChild(dot);

      row.appendChild(el('span', 'w-name', s.repo));
      row.appendChild(el('span', 'spacer', ''));

      // 失败显示 ✕N 角标（一眼可见），否则显示最近活动时间
      if (s.status === 'failed' && s.failed_count > 0) {
        row.appendChild(el('span', 'w-badge', `✕ ${s.failed_count}`));
      } else {
        const time = relativeTime(s.rows?.[0]?.date || s.pushed_at);
        if (time) row.appendChild(el('span', 'w-time', time));
      }

      // 点击整行直达该仓库的 Actions 页（与托盘菜单行为一致）
      row.title = tStatus(s.status);
      row.addEventListener('click', () => {
        invoke('open_url', { url: `https://github.com/${s.repo}/actions` });
      });
      group.appendChild(row);
    }
    list.appendChild(group);
  }

  // 头部聚合状态（N/M 通过数）
  const overall = overallOf(snapshot.repos);
  const pill = document.getElementById('w-overall');
  pill.className = `pill ${overall.key === 'failed' ? 'red' : overall.key === 'running' ? 'orange' : overall.key === 'success' ? 'green' : overall.key === 'error' ? 'purple' : 'gray'}`;
  pill.querySelector('.dot').style.background = STATUS_COLOR[overall.key] || 'var(--gray)';
  const passed = snapshot.repos.filter((s) => s.status === 'success').length;
  document.getElementById('w-overall-text').textContent = snapshot.repos.length
    ? `${passed}/${snapshot.repos.length}`
    : tStatus(overall.key);
  const refreshBtn = document.getElementById('w-refresh-btn');
  refreshBtn.disabled = snapshot.refreshing;
  refreshBtn.classList.toggle('busy', snapshot.refreshing);
}

// ============ 拖动、图钉锁定与手动刷新 ============

// 图钉状态（持久化在 config.widget.pinned）：锁定后头部拖动手柄失效
let pinned = false;

function applyPinUI() {
  const btn = document.getElementById('w-pin-btn');
  if (!btn) return;
  btn.classList.toggle('pinned', pinned);
  btn.title = pinned ? t('unpin') : t('pin');
  // 锁定即视为已知如何摆放，拖动引导一并收起
  if (pinned) document.getElementById('w-hint').classList.add('hidden');
}

function setupPin() {
  document.getElementById('w-pin-btn').innerHTML = icon('pin');
  document.getElementById('w-pin-btn').addEventListener('click', () => {
    pinned = !pinned;
    applyPinUI();
    invoke('widget_set_pinned', { pinned }).catch((e) => report(`widget_set_pinned failed: ${e}`));
  });
}

// 手动刷新：先乐观置 refreshing 让按钮立刻转起来——poller 要等整轮抓取完成才
// 广播 status-updated，不乐观更新的话点击后数秒内毫无反馈（wx：点了没感觉）
function triggerRefresh() {
  snapshot = { ...snapshot, refreshing: true };
  renderWidget();
  invoke('refresh_now').catch((e) => report(`refresh_now failed: ${e}`));
}

// 拖动引导：Rust 侧判定组件处于默认位置（屏幕中央，首次开启或位置失效回退）
// 时才显示；首次拖动后隐藏，之后位置已记忆不再出现
function setupDragHint() {
  const hint = document.getElementById('w-hint');
  invoke('widget_at_default')
    .then((atDefault) => {
      if (!atDefault || pinned) return;
      hint.classList.remove('hidden');
      let dismissed = false;
      getCurrentWindow().onMoved(() => {
        if (dismissed) return;
        dismissed = true;
        hint.classList.add('hidden');
      });
    })
    .catch((e) => report(`widget_at_default failed: ${e}`));
}

function setupDrag() {
  // 头部空白区域拖动窗口（按钮点击不触发）；不用 data-tauri-drag-region 是
  // 为了避开其双击最大化行为。图钉锁定后拖动手柄整体失效（Rust 侧
  // set_movable(false) 双保险，防其他路径触发移动）
  const header = document.getElementById('widget-header');
  header.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return;
    if (e.target.closest('button')) return;
    if (pinned) return;
    getCurrentWindow().startDragging();
  });
}

function hideCtxMenu() {
  document.getElementById('w-ctx-menu').classList.add('hidden');
}

function setupCtxMenu() {
  const menu = document.getElementById('w-ctx-menu');
  const root = document.getElementById('widget-root');

  root.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    menu.classList.remove('hidden');
    // 弹层定位在光标处，且不超出窗口
    const r = root.getBoundingClientRect();
    const mw = menu.offsetWidth || 150;
    const mh = menu.offsetHeight || 110;
    const x = Math.min(e.clientX - r.left, r.width - mw - 6);
    const y = Math.min(e.clientY - r.top, r.height - mh - 6);
    menu.style.left = `${Math.max(4, x)}px`;
    menu.style.top = `${Math.max(4, y)}px`;
  });

  // 点击任意处/失焦收起
  document.addEventListener('mousedown', (e) => {
    if (!e.target.closest('#w-ctx-menu')) hideCtxMenu();
  });
  window.addEventListener('blur', hideCtxMenu);

  document.getElementById('w-ctx-refresh').addEventListener('click', () => {
    hideCtxMenu();
    triggerRefresh();
  });
  document.getElementById('w-ctx-panel').addEventListener('click', () => {
    hideCtxMenu();
    invoke('show_panel_cmd');
  });
  document.getElementById('w-ctx-close').addEventListener('click', () => {
    hideCtxMenu();
    invoke('set_widget_enabled', { enable: false });
  });

  // 头部 ⋯ 按钮复用同一菜单，弹在按钮下方。
  // 用 mousedown + stopPropagation：document 级 mousedown 收起逻辑先于 click 触发，
  // 若走 click 会在「收起→展开」之间打架，导致菜单开着时点按钮永远关不上
  document.getElementById('w-menu-btn').addEventListener('mousedown', (e) => {
    e.stopPropagation();
    const hidden = menu.classList.contains('hidden');
    if (hidden) {
      menu.classList.remove('hidden');
      const r = document.getElementById('widget-root').getBoundingClientRect();
      const btn = e.currentTarget.getBoundingClientRect();
      menu.style.left = `${Math.max(4, btn.right - r.left - (menu.offsetWidth || 150))}px`;
      menu.style.top = `${btn.bottom - r.top + 4}px`;
    } else {
      menu.classList.add('hidden');
    }
  });
}

// ============ 启动 ============

async function init() {
  try {
    const cfg = await invoke('get_config');
    lang = cfg.lang || detectLang();
    pinned = !!(cfg.widget && cfg.widget.pinned);
  } catch (e) {
    lang = detectLang();
  }
  applyI18n();
  document.getElementById('w-refresh-btn').innerHTML = icon('refresh-cw');
  document.getElementById('w-refresh-btn').addEventListener('click', triggerRefresh);
  document.getElementById('w-menu-btn').innerHTML = icon('more-h');
  setupPin();

  try {
    snapshot = await invoke('get_snapshot');
  } catch (e) {
    report(`widget init snapshot failed: ${e}`);
  }
  renderWidget();

  listen('status-updated', (event) => {
    snapshot = event.payload;
    renderWidget();
  });

  // 兜底轮询：桌面层窗口被遮挡/暂停时事件可能丢失，60 秒对表一次
  setInterval(async () => {
    if (document.hidden) return;
    try {
      const fresh = await invoke('get_snapshot');
      if (JSON.stringify(fresh) !== JSON.stringify(snapshot)) {
        snapshot = fresh;
        renderWidget();
      }
    } catch (e) { /* 下轮再试 */ }
  }, 60000);

  setupDrag();
  setupDragHint();
  setupCtxMenu();
}

init();
