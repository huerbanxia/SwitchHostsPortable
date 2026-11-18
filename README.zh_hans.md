# SwitchHosts Portable（绿色版）

> **免安装 · 绿色便携** · 数据随程序走，整个目录可整体移动 / 复制 / 备份。
>
> 🤖 本绿色（便携）化改造由 **AI 辅助完成**（在 DeepSeek 语音会话中逐步实现）。

本仓库是对 [oldj/SwitchHosts](https://github.com/oldj/SwitchHosts) 的**绿色（便携）版**重新打包。解压即用，不写注册表、不在用户目录（`%APPDATA%` / `%LOCALAPPDATA%` / `%TEMP%`）遗留任何数据。

## 目录结构

```
SwitchHostsPortable/
├── help.html                     ← 简要说明
├── App/                          ← 只读程序体
│   ├── AppInfo/                  ← 便携元数据 + 图标
│   └── SwitchHosts/
│       ├── SwitchHostsPortable.exe  ← 主程序（双击启动）
│       └── .portable             ← 便携模式标记（勿删）
└── Data/                         ← 一切可写数据（PortableApps 规范）
    ├── SwitchHosts/              ← hosts 数据 / 配置 / 历史
    ├── Logs/                     ← 应用日志
    └── WebView2/                 ← WebView2 浏览器缓存（可再生，可删）
```

- **全部可写数据（含日志、WebView2 缓存）都落在程序目录 `Data/` 内**，用户目录零残留。
- **备份 = 复制整个目录**；到新机器 / 新位置解压即用，重装系统不丢配置。
- **单独程序**：只有一个 `App/SwitchHosts/SwitchHostsPortable.exe`，可在桌面建快捷方式。
- 改写系统 hosts 文件仍需管理员权限（Windows 机制），程序会照常弹出 UAC 授权。

## 下载（正式版）

前往 [GitHub Releases](https://github.com/huerbanxia/SwitchHostsPortable/releases) 下载
最新的 `SwitchHostsPortable.zip` 压缩包，解压到任意非系统盘目录，双击
`App/SwitchHosts/SwitchHostsPortable.exe` 即可使用。

## 构建绿色版

```bash
npm install
npm run tauri:build:portable   # 产出 dist/SwitchHostsPortable/
```

详见 [docs/portable.zh_cn.md](docs/portable.zh_cn.md)。

## 鸣谢

- **原项目 [oldj/SwitchHosts](https://github.com/oldj/SwitchHosts) 及其贡献者**——本便携版基于其重新打包。
- **Tauri**（[tauri.app](https://tauri.app/)）与 **React** 团队。

---

---

# 以下是原版（SwitchHosts）说明

<div align="center" markdown="1">
  <sup>Special thanks to:</sup>
  <br>
  <a href="https://go.warp.dev/SwitchHosts">
    <img alt="Warp sponsorship" width="400" src="https://github.com/user-attachments/assets/bb4a0222-12bf-4c79-bb80-a8ed4672b801" />
  </a>

### [Warp, the intelligent terminal for developers](https://go.warp.dev/SwitchHosts)
[Available for MacOS, Linux, & Windows](https://go.warp.dev/SwitchHosts)<br>

</div>

## SwitchHosts

轻松管理和切换 hosts 文件，基于 [Tauri](https://tauri.app/) 和 [React](https://react.dev/) 构建。

[项目主页](https://switchhosts.app) · [English](README.md) · [繁體中文](README.zh_hant.md) · [Polski](README.pl.md) · [Türkçe](README.tr.md)

<img src="./screenshots/switchhosts-light.png" alt="SwitchHosts 截图" width="978">

### 功能特性

- 管理本地和远程 hosts，支持组合与文件夹
- 从主窗口或系统托盘快速切换 hosts
- 支持语法高亮、查找和替换
- 自动刷新远程 hosts，导入与导出备份

### 下载

前往 [GitHub Releases](https://github.com/oldj/SwitchHosts/releases) 下载，或在 Windows 上通过 [Chocolatey](https://community.chocolatey.org/packages/switchhosts) 安装：`choco install switchhosts`。

### 文档与开发

编辑 hosts、数据备份、本地 HTTP API 和构建环境要求详见[使用与开发指南](docs/guide.zh_hans.md)。

```bash
npm install
npm run tauri:dev    # 开发
npm run tauri:build  # 生产构建
```

### 版权

SwitchHosts 是一个免费开源软件，基于 Apache-2.0 协议发布。
