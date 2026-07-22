# Changelog

All notable changes to Hanono will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1](https://github.com/eproduce/hanono/compare/v0.5.0...v0.5.1) (2026-07-22)


### Bug Fixes

* 修复列表循环/单曲循环/随机播放在最后一首失效的bug ([1e0e734](https://github.com/eproduce/hanono/commit/1e0e73449393cf129ff80be5ecabf538300b270b))

## [0.4.4](https://github.com/eproduce/hanono/compare/v0.4.3...v0.4.4) (2026-07-21)


### Features

* v0.5.0 - 歌词展示与在线搜索 ([d8d7938](https://github.com/eproduce/hanono/commit/d8d7938059cbc361fc4c095541748eb3ceb7e512))


### Bug Fixes

* 修复音源切换变音 + 新增歌词缓存刷新功能 ([21977d8](https://github.com/eproduce/hanono/commit/21977d8219e8fac4b3cb8ff55bf67553ca6c9f02))

## [0.4.3](https://github.com/eproduce/hanono/compare/v0.4.2...v0.4.3) (2026-07-20)


### Features

* 波形图性能大幅优化 + 窗口布局重构 + 专辑封面提取 ([621736a](https://github.com/eproduce/hanono/commit/621736abf051aae87d3251a5ee97933decc1f515))


### Performance Improvements

* 波形图 ffmpeg 快速生成 + 磁盘缓存 + 窗口布局优化 ([3c5bb60](https://github.com/eproduce/hanono/commit/3c5bb60799c0b27baa6c4fd379d024ecab4db3cb))

## [0.4.2](https://github.com/eproduce/hanono/compare/hanono-v0.4.1...hanono-v0.4.2) (2026-07-20)


### Features

* FFmpeg 集成 — 波形图、音频信息、格式转换、裁剪、音量标准化 ([afd7525](https://github.com/eproduce/hanono/commit/afd7525983f10a35c9b4a917d902ffe26d372ea3))
* Windows 绿色便携版 (zip)，免安装解压即用 ([66e86c3](https://github.com/eproduce/hanono/commit/66e86c3bb996c8ab34a8f6e8c46014abd55ea008))
* 麒麟 V10 离线部署脚本，无 FUSE 解压 AppImage 启动 ([71a86df](https://github.com/eproduce/hanono/commit/71a86dfc5a79f9bb1895650b58038b50e8e18276))


### Bug Fixes

* release-please tag 格式修复，去掉 @ 分隔符 ([91ae705](https://github.com/eproduce/hanono/commit/91ae70513e8082fe186c0dfc8aa5bec2de172ad5))
* 修复多首歌同时播放 + 音量跳变 + 新增专辑封面提取 ([c2bf5a7](https://github.com/eproduce/hanono/commit/c2bf5a7767b7f1f2f079eb20061e84fb8361346a))
* 关闭→最小化，移除 Reopen 依赖，兼容所有 Tauri 2.x 和全平台 ([51db9cf](https://github.com/eproduce/hanono/commit/51db9cf09c9e17b287b9ba92f19f2c79111d79dc))
* 恢复 RunEvent::Reopen (dock点击恢复) + 图标圆角 ([9eee2e9](https://github.com/eproduce/hanono/commit/9eee2e9bfaa27c7b5a9bdce12d3f5df9ac043630))
* 移除 RunEvent::Reopen，改用 WindowEvent::Focused 处理 dock 恢复 ([06640ad](https://github.com/eproduce/hanono/commit/06640addce4e45a01a7eab29905b373195973d65))
* 精确锁定 Tauri 2.11.5 确保 Linux/Windows 兼容 Reopen ([7ae33d2](https://github.com/eproduce/hanono/commit/7ae33d2e0a3181451ad88dea0e1d7f7b64cbd973))
* 锁定 Tauri &gt;= 2.2 修复 CI Reopen 编译错误 ([d430a54](https://github.com/eproduce/hanono/commit/d430a54417cd4d7f90b2dc118baf676ea0b2f858))

## [0.4.1](https://github.com/eproduce/hanono/compare/hanono@v0.4.0...hanono@v0.4.1) (2026-07-20)


### Features

* FFmpeg 集成 — 波形图、音频信息、格式转换、裁剪、音量标准化 ([afd7525](https://github.com/eproduce/hanono/commit/afd7525983f10a35c9b4a917d902ffe26d372ea3))
* Windows 绿色便携版 (zip)，免安装解压即用 ([66e86c3](https://github.com/eproduce/hanono/commit/66e86c3bb996c8ab34a8f6e8c46014abd55ea008))
* 麒麟 V10 离线部署脚本，无 FUSE 解压 AppImage 启动 ([71a86df](https://github.com/eproduce/hanono/commit/71a86dfc5a79f9bb1895650b58038b50e8e18276))


### Bug Fixes

* 修复多首歌同时播放 + 音量跳变 + 新增专辑封面提取 ([c2bf5a7](https://github.com/eproduce/hanono/commit/c2bf5a7767b7f1f2f079eb20061e84fb8361346a))
* 关闭→最小化，移除 Reopen 依赖，兼容所有 Tauri 2.x 和全平台 ([51db9cf](https://github.com/eproduce/hanono/commit/51db9cf09c9e17b287b9ba92f19f2c79111d79dc))
* 恢复 RunEvent::Reopen (dock点击恢复) + 图标圆角 ([9eee2e9](https://github.com/eproduce/hanono/commit/9eee2e9bfaa27c7b5a9bdce12d3f5df9ac043630))
* 移除 RunEvent::Reopen，改用 WindowEvent::Focused 处理 dock 恢复 ([06640ad](https://github.com/eproduce/hanono/commit/06640addce4e45a01a7eab29905b373195973d65))
* 精确锁定 Tauri 2.11.5 确保 Linux/Windows 兼容 Reopen ([7ae33d2](https://github.com/eproduce/hanono/commit/7ae33d2e0a3181451ad88dea0e1d7f7b64cbd973))
* 锁定 Tauri &gt;= 2.2 修复 CI Reopen 编译错误 ([d430a54](https://github.com/eproduce/hanono/commit/d430a54417cd4d7f90b2dc118baf676ea0b2f858))

## 0.4.0 (2026-07-13)

### Added
- 🎛️ **音效增强系统**：7 种 EQ 预设（默认/流行/摇滚/爵士/古典/人声/低音）
- 5段图形均衡器（60Hz/250Hz/1kHz/4kHz/8kHz），±12dB 手动调节
- 低音增强（0～15dB）、3D 环绕声场、卷积混响
- Web Audio API 完整音频管线
- 音效模式标签实时显示在曲目信息栏
- 关闭窗口 → 最小化，Dock 点击恢复

### Changed
- 图标重新设计：紫色渐变 + 圆角背景
- CI/CD 全平台构建：macOS Intel / Apple Silicon / Windows / Linux
- release-please 自动版本管理 + CHANGELOG 生成

### Fixed
- 音效面板滑块操作崩溃（AudioContext 未初始化）
- Web Audio 接管后音量调节失效
- Linux / Windows CI 编译兼容性
- macOS 交叉编译产物路径

## 0.1.0 (2026-07-10)

### Added
- 沉浸式本地音乐播放器 (Tauri 2 + Vue 3 + Rust)
- 多格式支持：FLAC / MP3 / WAV / OGG / AAC / M4A 等
- 拖拽导入 + SQLite 持久化播放列表
- 封面动效：旋转唱片 / 呼吸灯 / 流光光晕
- 歌词显示（.lrc 自动匹配）
- 睡眠定时器、播放速度控制
- Mini 播放器模式
- 系统托盘 + macOS 菜单栏集成
- Media Session API（Now Playing）
- 收藏夹 + 播放历史
- 右键菜单 + 快捷键面板
