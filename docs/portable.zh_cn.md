# SwitchHosts 绿色版（Portable）使用与构建说明

本文介绍如何构建和使用**绿色版（免安装 / portable）** SwitchHosts。绿色版把
所有可写数据放进程序目录内的 `Data/`，不写用户目录、不写注册表，整个目录拷
贝走即是完整的备份与迁移。

> 安装版（`tauri build` / NSIS 安装包）行为与官方一致，不受本文档影响。
> 绿色版与安装版可同时存在，二者数据目录互不干扰。

---

## 目录结构

绿色版构建脚本会生成下面的目录（`App/` 放只读程序体，`Data/` 放可写用户数据，
沿用 PortableApps 风格，与常见的便携软件布局一致）：

```
SwitchHostsPortable/
├── SwitchHostsPortable.exe     ← 双击此文件启动（主程序副本）
├── help.html                   ← 简要说明（浏览器打开）
├── App/                        ← 只读程序体
│   ├── AppInfo/
│   │   ├── appinfo.ini         ← 便携元数据
│   │   └── appicon.ico
│   └── SwitchHosts/
│       ├── SwitchHosts.exe     ← 真正的应用二进制
│       └── .portable           ← 便携模式标记文件（勿删）
└── Data/                       ← 可写用户数据
    ├── Logs/                   ← 应用日志
    ├── SwitchHosts/            ← 数据根：hosts 列表、配置、窗口状态、历史
    └── WebView2/               ← WebView2 浏览器缓存（已重定向到此处）
```

- **不写系统/用户目录**：不会在 `%APPDATA%`、`%LOCALAPPDATA%`、`%TEMP%`、
  `~/.SwitchHosts` 写入任何东西（唯一例外是系统 hosts 文件本身，见下文）。
  - 应用日志 → `Data/Logs`
  - 配置与 hosts 数据 → `Data/SwitchHosts`
  - **WebView2 浏览器缓存（Cookie、GPU 缓存、Local Storage 等）→ `Data/WebView2`**
    —— WebView2 默认会把浏览器 profile 写到
    `%LOCALAPPDATA%\<identifier>\EBWebView`，绿色版通过
    `WebviewWindowBuilder::data_directory` 把它重定向进 `Data/WebView2`，
    因此连浏览器缓存也算在程序目录内。
- **备份 = 复制整个目录** `SwitchHostsPortable/`，到新机器/新位置解压即用。

---

## 如何判断是否进入便携模式

两种信号任一命中即进入便携模式（二者取或）：

1. 启动前设置环境变量 `SWITCHHOSTS_PORTABLE`（任意非空值）。
2. `App/SwitchHosts/SwitchHosts.exe` 或其上级目录中存在 `.portable` 标记文件。

绿色版构建脚本天然放好了 `.portable` 标记，所以解压后双击即自动进入便携模式，
无需手动设置环境变量。

---

## 构建绿色版

前置要求：Node 24+、Rust（MSVC）、Visual Studio Build Tools（含
WebView2 + NSIS 依赖随构建自动处理，见 Tauri 官方要求）。

```bash
# 1. 安装依赖
npm ci

# 2. 形式化检查 + 单元测试（可选但推荐）
npm run typecheck
npm run test:unit

# 3. 构建绿色版（等价于：tauri build --bundles none && npm run build:portable）
npm run tauri:build:portable
```

产物输出到 `dist/portable/SwitchHostsPortable/`。

> 说明：`--bundles none` 只生成可执行的 release 二进制（不生成 NSIS/MSI
> 安装包），随后由 `scripts/build-portable.mjs` 编排成上文的 `App/ + Data/` 布局。

也可用安装版命令产出安装包：`npm run tauri:build`（不受绿色版改动影响）。

---

## 使用场景：配置备份 / 同步 / 重装系统不丢

这是本绿色版最典型的用法。数据根固定为 `<绿色版>/Data/SwitchHosts`，因此：

- **用你现有的同步软件同步 `Data/SwitchHosts` 子目录**即可自动备份配置，例如：
  - **Synology Drive**：把同步文件夹设置到绿色版 `Data/SwitchHosts`（或直接同步整个
    绿色版目录），改动即时同步。
  - **FreeFileSync / 其它**：按需做"单程镜像"或双向比较，把 `Data/SwitchHosts`
    备份到你自己的备份根（如 `D:\SynologyDrive\...\SwitchHosts`）。
- **重装系统后恢复**：
  1. 绿色版目录放在**非系统盘**（如 `D:\`）。
  2. 系统盘重装不影响 `D:\SwitchHostsPortable`。
  3. 重装后到绿色版目录双击 `SwitchHostsPortable.exe` 即可，数据仍在
     `Data/SwitchHosts`，**无需重新指定任何配置路径**（不再像安装版那样依赖
     `%APPDATA%` 里的指针文件——重装一清就丢）。

> 对比：安装版把"配置位置"记在 `%APPDATA%\net.oldj.switchhosts\data_dir.json`
> 指针里，重装系统后该指针丢失，程序会回落默认目录。绿色版把数据根固定在
> `Data/SwitchHosts`，从根本上不存在"配置位置丢失"。

---

## 常见问题（FAQ）

### Q：改 hosts 文件还需要管理员权限吗？
需要。改写系统 hosts（`C:\Windows\System32\drivers\etc\hosts`）是操作系统机制，
绿色版同样会弹出 UAC 提权窗口。这是唯一需要提权的动作。

### Q：绿色版能不能开机自启？
不能。绿色版刻意关闭开机自启（不写用户注册表 Run 键 / LaunchAgent）。如确需自启，
可用 Windows"启动"文件夹放一个快捷方式指向 `SwitchHostsPortable.exe`。

### Q：绿色版会自动更新吗？
不会。绿色版关闭了在线自动更新检测。升级 = 用新版本的整个目录替换旧目录
（保留旧 `Data/` 即可）。

### Q：我之前用安装版，怎么把数据带到绿色版？
在安装版里"导出备份"，再到绿色版"导入备份"。也可以直接把安装版的数据目录
（`~/.SwitchHosts` 或你自定义的位置）下的文件同步/复制到绿色版 `Data/SwitchHosts`。

### Q：运行环境要装 WebView2 吗？
需要 Windows 10/11 通常自带 WebView2 运行时。若极旧的系统没有，安装
[Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
即可。

### Q：绿色版能放在只读位置吗？
不建议放 `C:\Program Files` 等只读目录。若数据目录不可写，程序会进入数据目录
恢复流程提示。建议放普通可写目录（如 `D:\Tools\SwitchHostsPortable`）。

### Q：桌面图标样式和 AppInfo 是做什么的？
`App/AppInfo`（`appinfo.ini` + `appicon.ico`）是便携软件的通用元数据，
便于某些便携菜单/收藏工具识别图标与版本。

---

## 与主线（安装版）的关系

绿色版改动遵循"运行时开关隔离"原则：

| 概念 | 安装版（主线） | 绿色版 |
|---|---|---|
| 默认数据根 | `~/.SwitchHosts` | `<绿色版>/Data/SwitchHosts` |
| 自定义目录指针 | `%APPDATA%` 下 | 不使用（数据根固定） |
| 更换/重置数据目录 | 支持 | 界面隐藏（数据根固定） |
| 日志 | 平台日志目录 | `Data/Logs` |
| 开机自启 | 支持 | 隐藏并关闭 |
| 自动更新 | 支持 | 关闭 |
| v4 旧数据迁移 | 读 `%APPDATA%` | 从空开始（用导入备份继承） |

安装版逻辑完全保留：`tauri build`（NSIS 安装包）行为不变，既有单元测试仍然通过。