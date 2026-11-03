// Glint landing — 双语 i18n，模式与 tauri/ui/app.js 同源（data-i18n + 字典 + navigator.language 检测）
// 差异：无 config 文件，持久化用 localStorage

const I18N = {
  en: {
    'meta.title': 'Glint — CI status at a glance',

    'nav.features': 'Features',
    'nav.quickstart': 'Quick start',
    'nav.download': 'Download',

    'hero.tagline': 'CI status at a glance',
    'hero.sub': 'A tray-resident GitHub CI status dashboard — no more opening the browser to check if CI passed. When something fails, drill down to the failing job ▸ step without ever opening a web page.',
    'hero.quote': '“Push, switch away, and relax: it will shout when things go red, and open it to fix.”',
    'hero.download': 'Download',
    'hero.star': 'Star on GitHub',
    'hero.meta': 'Free & open source (MIT) · macOS · Linux · Windows',
    'hero.install': 'Builds are not notarized yet — on first launch, right-click the app and choose Open.',

    'mock.failing': '1 failing',
    'mock.onDefault': 'default branch',
    'mock.duration': 'failed · 2m 14s',
    'mock.viewlog': 'View log',
    'mock.copyerror': 'Copy error',
    'mock.passing': 'All checks passed',
    'mock.running': 'CI running',
    'mock.now': 'just now',

    'tray.pass': 'All passing',
    'tray.fail': 'Failures',
    'tray.running': 'Running',
    'tray.noci': 'No CI',
    'tray.error': 'API error',
    'tray.caption': 'One dot in your menu bar — aggregated worst status across every repo you watch.',

    'features.title': 'Why Glint',
    'features.sub': 'Everything you need to stop babysitting CI in a browser tab.',
    'f1.t': 'Glanceable tray',
    'f1.d': 'One dot for every repo you monitor — aggregated worst status in five states: green, red with a failure count, orange while running, gray with no CI, purple on API errors.',
    'f2.t': 'Actionable failures',
    'f2.d': 'Drill straight to the failing job ▸ step with duration and a direct log link — copy the error excerpt without ever opening a browser.',
    'f3.t': 'Notifications that behave',
    'f3.d': 'An instant alert when a build fails, a gentle note when it recovers — each toggleable, in your language. Clicking one brings the panel up.',
    'f4.t': 'Zero-config sign-in',
    'f4.d': "Authorize once through the browser with GitHub's Device Flow — the same as gh auth login. No tokens to create or paste; the token lives only in your system keychain.",
    'f5.t': 'Private by design',
    'f5.d': "Runs entirely on your machine: no server, no account, no telemetry. ETag conditional requests keep you comfortably below GitHub's rate limits.",
    'f6.t': 'Lightweight & bilingual',
    'f6.d': 'Built on Tauri 2 with the system WebView — a tiny footprint, no Electron. The UI speaks English and Chinese, following your system language.',

    'qs.title': 'Up and running in a minute',
    'qs.sub': 'No config files, no token juggling.',
    'qs1.t': 'Launch',
    'qs1.d': 'Open Glint — a dot appears in your menu bar.',
    'qs2.t': 'Sign in',
    'qs2.d': 'Authorize once through GitHub in your browser.',
    'qs3.t': 'Pick repos',
    'qs3.d': 'Tick the repositories to watch from a fetched checklist.',
    'qs4.t': 'Relax',
    'qs4.d': 'The tray reflects CI from now on; failures pop a notification.',

    'plat.title': 'Platforms',
    'plat.available': 'Available now',
    'plat.macos': 'Universal binary (Apple Silicon + Intel) via GitHub Releases.',
    'plat.linux': 'deb / AppImage packages via GitHub Releases.',
    'plat.win': 'MSI / NSIS installer via GitHub Releases.',

    'footer.cta': 'Stop checking CI. Start glancing at it.',
    'footer.license': 'Released under the MIT License.',
    'footer.note': 'Glint is an independent open-source project and is not affiliated with GitHub, Inc.',
  },

  zh: {
    'meta.title': 'Glint — 一眼即知 CI 状态',

    'nav.features': '功能',
    'nav.quickstart': '快速上手',
    'nav.download': '下载',

    'hero.tagline': '一眼即知 CI 状态',
    'hero.sub': '驻守菜单栏的 GitHub CI 状态仪表盘——不用再开网页确认 CI 是否通过。失败时直接下钻到具体的 job ▸ step，全程无需打开浏览器。',
    'hero.quote': '“Push 后放心切走干别的：红了会叫你，点开就能修。”',
    'hero.download': '下载安装包',
    'hero.star': '到 GitHub 点个 Star',
    'hero.meta': '免费开源（MIT）· macOS · Linux · Windows',
    'hero.install': '当前构建尚未公证——首次启动请右键点击应用并选择「打开」。',

    'mock.failing': '1 个失败',
    'mock.onDefault': '默认分支',
    'mock.duration': '失败 · 2m 14s',
    'mock.viewlog': '查看日志',
    'mock.copyerror': '复制错误',
    'mock.passing': '全部检查通过',
    'mock.running': 'CI 运行中',
    'mock.now': '刚刚',

    'tray.pass': '全部通过',
    'tray.fail': '有失败',
    'tray.running': '运行中',
    'tray.noci': '无 CI',
    'tray.error': 'API 出错',
    'tray.caption': '菜单栏上一颗圆点——聚合你所有关注仓库的最差状态。',

    'features.title': '为什么选择 Glint',
    'features.sub': '把「盯着浏览器标签页等 CI」彻底省掉。',
    'f1.t': '一眼可知的托盘',
    'f1.d': '所有关注仓库聚合出一颗圆点，五态展示：绿色全过、红色带失败数、橙色运行中、灰色无 CI、紫色 API 出错。',
    'f2.t': '失败可行动',
    'f2.d': '直接定位到失败的 job ▸ step，附耗时与日志直链——复制错误摘录，不用打开浏览器。',
    'f3.t': '懂分寸的通知',
    'f3.d': '构建失败立刻提醒，恢复时轻声告知——各自可开关，跟随你的语言。点通知即可唤起面板。',
    'f4.t': '零配置登录',
    'f4.d': '通过浏览器用 GitHub Device Flow 授权一次——和 gh auth login 同款。无需创建或粘贴 token，它只存在系统钥匙串里。',
    'f5.t': '隐私优先',
    'f5.d': '完全本地运行：无服务器、无账号、无遥测。ETag 条件请求让你远低于 GitHub 限流阈值。',
    'f6.t': '轻量 · 双语',
    'f6.d': '基于 Tauri 2 与系统 WebView——体积极小，没有 Electron。界面支持中英双语，跟随系统语言。',

    'qs.title': '一分钟上手',
    'qs.sub': '无配置文件，无需折腾 token。',
    'qs1.t': '启动',
    'qs1.d': '打开 Glint——菜单栏出现一颗圆点。',
    'qs2.t': '登录',
    'qs2.d': '在浏览器里完成一次 GitHub 授权。',
    'qs3.t': '选仓库',
    'qs3.d': '从拉取的列表里勾选要关注的仓库。',
    'qs4.t': '放心干活',
    'qs4.d': '从此托盘即 CI 状态，失败会弹通知。',

    'plat.title': '平台支持',
    'plat.available': '现已可用',
    'plat.macos': '通用二进制（Apple Silicon + Intel），经 GitHub Releases 分发。',
    'plat.linux': 'deb / AppImage 安装包，经 GitHub Releases 分发。',
    'plat.win': 'MSI / NSIS 安装器，经 GitHub Releases 分发。',

    'footer.cta': '别再盯着 CI 等，瞥一眼就好。',
    'footer.license': '基于 MIT 协议开源。',
    'footer.note': 'Glint 是独立的开源项目，与 GitHub, Inc. 无隶属关系。',
  },
};

let lang = localStorage.getItem('glint-site-lang');
if (lang !== 'zh' && lang !== 'en') {
  lang = (navigator.language || 'en').toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

function t(key) {
  return I18N[lang][key] ?? I18N.en[key] ?? key;
}

function applyLang() {
  document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en';
  document.title = t('meta.title');
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  document.getElementById('lang-toggle').textContent = lang === 'zh' ? 'EN' : '中文';
}

document.getElementById('lang-toggle').addEventListener('click', () => {
  lang = lang === 'zh' ? 'en' : 'zh';
  localStorage.setItem('glint-site-lang', lang);
  applyLang();
});

applyLang();
