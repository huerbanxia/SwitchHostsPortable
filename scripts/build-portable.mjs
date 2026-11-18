#!/usr/bin/env node
/**
 * Package a portable ("green") SwitchHosts build for Windows.
 *
 * The Tauri bundle targets (NSIS / MSI) install into the user profile and
 * write to the OS. For a green build we instead lay out the PortableApps
 * style directory and point all *writable* data at a per-directory `Data/`
 * folder. At runtime the app detects portable mode via the `SWITCHHOSTS_PORTABLE`
 * env var or a `.portable` marker and stores its data under `Data/SwitchHosts`
 * and logs under `Data/Logs` (see `src-tauri/src/storage/portable.rs`).
 *
 * Result layout (PortableApps convention: App = read-only, Data = everything writable):
 *
 *   dist/SwitchHostsPortable/
 *   ├── help.html                        <- brief portable README (open in browser)
 *   ├── App/                             <- read-only program body
 *   │   ├── AppInfo/
 *   │   │   ├── appinfo.ini              <- portable metadata
 *   │   │   └── appicon.ico              <- desktop icon
 *   │   └── SwitchHosts/
 *   │       ├── SwitchHostsPortable.exe  <- actual Tauri app binary (renamed)
 *   │       └── .portable                <- marker file that flips portable mode on
 *   └── Data/                            <- everything writable lives here
 *       ├── SwitchHosts/                 <- data root (manifest, entries, config, state, history)
 *       ├── Logs/                        <- application logs
 *       └── WebView2/                    <- WebView2 browser profile (GPU cache, cookies, localStorage)
 *
 * Only one app binary exists, at App/SwitchHosts/SwitchHostsPortable.exe —
 * the portable detection (is_portable) recognises this location via the App/
 * + Data layout and the .portable marker, so there is no duplicated exe at the
 * green root. The exe filename is renamed from the Tauri product name
 * (SwitchHosts.exe) to SwitchHostsPortable.exe for distribution; productName
 * itself is left untouched so the app identifiers/process name are unchanged.
 *
 * Run after `tauri build --no-bundle` via:
 *   npm run tauri:build:portable
 *
 * On non-Windows platforms this script warns and exits (portable layout is
 * Windows-focused), so it can be wired into cross-platform CI safely.
 */

import { existsSync, mkdirSync, copyFileSync, writeFileSync, readFileSync } from 'fs'
import { stat } from 'fs/promises'
import { dirname, join } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')
const version = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version

if (process.platform !== 'win32') {
  // The PortableApps-style layout is Windows-centric; on other platforms
  // (e.g. cross-platform CI) we warn and skip rather than fail the pipeline.
  console.warn('[portable] portable packaging is Windows-only; skipping on this platform.')
  process.exit(0)
}

// ---- locate the built app binary ------------------------------------------

const possibleApps = [
  join(root, 'src-tauri', 'target', 'release', 'SwitchHosts.exe'),
  join(root, 'src-tauri', 'target', 'release', 'switchhosts.exe'),
]
const appExe = possibleApps.find((p) => existsSync(p))
if (!appExe) {
  console.error(
    `[portable] release build not found. Build the app first, e.g.\n` +
      `  npm run tauri:build:portable\n` +
      `Looked for: ${possibleApps.join(' | ')}`,
  )
  process.exit(1)
}

// ---- assemble the portable layout ------------------------------------------

const MAIN_EXE_NAME = 'SwitchHostsPortable.exe' // renamed on copy; productName untouched
const distRoot = join(root, 'dist')
const green = join(distRoot, 'SwitchHostsPortable')
const appDir = join(green, 'App')
const productDir = join(appDir, 'SwitchHosts')
const appInfoDir = join(appDir, 'AppInfo')
const dataDir = join(green, 'Data')
const logDir = join(dataDir, 'Logs')
const dataRoot = join(dataDir, 'SwitchHosts')
const webviewDir = join(dataDir, 'WebView2')

for (const d of [productDir, appInfoDir, dataRoot, logDir, webviewDir]) {
  mkdirSync(d, { recursive: true })
}

// The real Tauri binary (renamed to MAIN_EXE_NAME) plus the portable marker
// file (so the backend knows to scope all data under `Data/`).
copyFileSync(appExe, join(productDir, MAIN_EXE_NAME))
writeFileSync(join(productDir, '.portable'), '')

// Portable metadata (AppInfo) + desktop icon.
const appIcon = join(root, 'src-tauri', 'icons', 'icon.ico')
if (existsSync(appIcon)) {
  copyFileSync(appIcon, join(appInfoDir, 'appicon.ico'))
}
writeFileSync(
  join(appInfoDir, 'appinfo.ini'),
  [
    '[Format]',
    'Type=Portable',
    '',
    '[Details]',
    `Name=SwitchHosts`,
    `AppVersion=${version}`,
    `DisplayVersion=${version}`,
    'Publisher=oldj & contributors',
    'Homepage=https://github.com/oldj/SwitchHosts',
    '',
    '[Control]',
    `Start=App/SwitchHosts/${MAIN_EXE_NAME}`,
  ].join('\r\n') + '\r\n',
)

// Brief offline help.
writeFileSync(
  join(green, 'help.html'),
  `<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>SwitchHosts 绿色版</title>
<style>body{font-family:system-ui,sans-serif;max-width:720px;margin:40px auto;line-height:1.6;color:#222}
code{background:#f2f2f2;padding:0 4px;border-radius:4px}li{margin:6px 0}</style>
<h2>SwitchHosts 绿色版（v${version}）</h2>
<ul>
<li><b>绿色运行</b>：无安装、无注册表项、不写用户目录。双击 \`App/SwitchHosts/SwitchHostsPortable.exe\` 即可使用（在桌面建一个快捷方式更方便）。</li>
<li><b>数据目录</b>：Hosts 数据在 <code>Data/SwitchHosts</code>，日志 <code>Data/Logs</code>，WebView2 缓存 <code>Data/WebView2</code>。备份/迁移 = 复制整个绿色版目录。</li>
<li><b>可清理</b>：<code>Data/WebView2</code> 是 WebView2 浏览器缓存（GPU 缓存、Cookie 等），属于可再生数据，<b>删除不影响你的 hosts 配置</b>。</li>
<li><b>开机自启</b>：绿色版不支持注册开机自启（避免写入用户注册表）。</li>
<li><b>自动更新</b>：绿色版关闭在线自动更新；"升级"＝用新版本整体替换本目录。</li>
<li><b>修改 hosts 文件</b>：改写系统 hosts 仍需管理员权限，程序会照常弹出 UAC 授权。</li>
<li><b>旧数据导入</b>：用安装版"导出备份"，再在本绿色版"导入备份"即可继承。</li>
</ul></html>
`,
)

const sizeExe = existsSync(join(productDir, MAIN_EXE_NAME))
  ? (await stat(join(productDir, MAIN_EXE_NAME))).size
  : 0

console.log(`[portable] SwitchHosts v${version} portable build assembled in:`)
console.log(`  ${green}`)
console.log(`  exe: App/SwitchHosts/${MAIN_EXE_NAME} (${(sizeExe / 1024 / 1024).toFixed(1)} MiB)`)
console.log(`  data: Data/SwitchHosts (created on first run)`)
console.log(`  logs: Data/Logs   cache: Data/WebView2`)