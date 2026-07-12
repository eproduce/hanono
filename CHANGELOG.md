# Changelog

All notable changes to Hanono will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2](https://github.com/eproduce/hanono/compare/hanono@v0.3.1...hanono@v0.3.2) (2026-07-12)


### Bug Fixes

* 关闭→最小化，移除 Reopen 依赖，兼容所有 Tauri 2.x 和全平台 ([51db9cf](https://github.com/eproduce/hanono/commit/51db9cf09c9e17b287b9ba92f19f2c79111d79dc))
* 恢复 RunEvent::Reopen (dock点击恢复) + 图标圆角 ([9eee2e9](https://github.com/eproduce/hanono/commit/9eee2e9bfaa27c7b5a9bdce12d3f5df9ac043630))
* 移除 RunEvent::Reopen，改用 WindowEvent::Focused 处理 dock 恢复 ([06640ad](https://github.com/eproduce/hanono/commit/06640addce4e45a01a7eab29905b373195973d65))
* 精确锁定 Tauri 2.11.5 确保 Linux/Windows 兼容 Reopen ([7ae33d2](https://github.com/eproduce/hanono/commit/7ae33d2e0a3181451ad88dea0e1d7f7b64cbd973))
* 锁定 Tauri &gt;= 2.2 修复 CI Reopen 编译错误 ([d430a54](https://github.com/eproduce/hanono/commit/d430a54417cd4d7f90b2dc118baf676ea0b2f858))

## 0.3.1 (2026-07-11)

### Added
- 🎛️ 音效增强系统：7 种 EQ 预设（默认/流行/摇滚/爵士/古典/人声/低音）
- 5段图形均衡器（60Hz/250Hz/1kHz/4kHz/8kHz），±12dB 手动调节
- 低音增强（0～15dB）、3D 环绕声场、卷积混响
- 音效模式标签实时显示
- Web Audio API 完整音频管线

### Fixed
- 音效面板滑块操作崩溃
- Web Audio 接管后音量调节失效
