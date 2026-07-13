# 🎵 Hanono

<div align="center">

**沉浸式本地音乐播放器**

Built with Tauri 2 + Vue 3 + Rust · Web Audio API 音效增强

[![Version](https://img.shields.io/badge/version-0.4.0-blue)](https://github.com/eproduce/hanono/releases)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/eproduce/hanono/releases)

</div>

## ✨ 功能

- 🎧 **多格式支持** — FLAC / MP3 / WAV / OGG / AAC / M4A / AIFF 等
- 🎛️ **音效增强** — 7 种 EQ 预设 + 5段图形均衡器 + 低音增强 + 3D环绕 + 混响
- 📋 **播放列表** — 拖拽导入、持久化存储、收藏夹、播放历史
- 🎨 **封面动效** — 旋转唱片 / 呼吸灯 / 流光光晕
- 📝 **歌词显示** — 自动匹配同目录 .lrc 文件
- ⏰ **睡眠定时** — 15/30/60 分钟自动暂停
- ⚡ **播放速度** — 0.5x ~ 2x 变速播放
- 🖥️ **系统集成** — macOS 菜单栏控制、系统托盘、Media Session
- 🪟 **Mini 模式** — 精简界面，适合边听边工作
- 🌐 **全平台** — macOS (Intel + Apple Silicon) / Windows / Linux

## 📦 下载

前往 [Releases](https://github.com/eproduce/hanono/releases) 下载最新版本：

| 平台 | 格式 | 说明 |
|------|------|------|
| macOS Intel | `.dmg` | macOS 13+ |
| macOS Apple Silicon | `.dmg` | macOS 13+ |
| Windows x64 | `.msi` / `.exe` | Windows 10+ |
| Linux x64 | `.AppImage` | **推荐**，自带依赖，适用所有发行版 |
| Linux x64 | `.deb` | Ubuntu 24.04+ / Debian 12+ |
| Linux x64 | `.rpm` | Fedora 39+ |

> ⚠️ **Linux 注意**：`.deb` / `.rpm` 需要 `libwebkit2gtk-4.1`，仅限较新发行版。老系统 / 麒麟 V10 / 离线环境请用 **AppImage**。

### 🔒 离线 / 内网 / 无 FUSE 环境部署（麒麟 V10 等）

```bash
# 1. 将 AppImage 复制到目标机器（U盘/内网传输）
# 2. 赋予执行权限
chmod +x Hanono_0.4.0_amd64.AppImage

# 3. 解压运行（无需 root，无需 FUSE，无需联网）
./Hanono_0.4.0_amd64.AppImage --appimage-extract
./squashfs-root/AppRun

# 或使用一键脚本
bash scripts/run-hanono.sh ./Hanono_0.4.0_amd64.AppImage
``` |

## 🛠️ 开发

### 环境要求

- [Node.js](https://nodejs.org/) >= 22
- [Rust](https://www.rust-lang.org/) >= 1.70
- macOS: Xcode Command Line Tools
- Linux: `libwebkit2gtk-4.1-dev` 等依赖
- Windows: Microsoft Visual Studio C++ Build Tools

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/eproduce/hanono.git
cd hanono

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

### 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | [Tauri 2](https://v2.tauri.app/) |
| 前端 | Vue 3 + TypeScript + Vite |
| 后端 | Rust |
| 音频处理 | Web Audio API (BiquadFilter / Convolver / StereoPanner) |
| 数据存储 | SQLite (rusqlite) |
| CI/CD | GitHub Actions + release-please |

## 📄 许可证

MIT © 2026 Hanono

