# 更新日志 / Changelog

## 0.3.0 — 2026-10-23

本分支首个正式发布版，基于上游 0.2.1。完整说明见 [v0.3.0 发布说明](docs/releases/v0.3.0.md)。

- 长按鼠标中键在触发位置呼出；松开后保留面板，左键选择软件。
- 长按时长 200～1000 ms 可调，默认 400 ms；短按回放，提前拖动交还应用。
- 固定软件四列网格、分页、Ctrl+P 和右键固定，保留搜索与路径补全。
- 设置和托盘暂停／恢复，暂停时释放全局快捷键与中键；前台 exe 名单自动禁用。
- 优先激活已有窗口、恢复最小化窗口、按最近使用选择；支持跳过复用的新开实例。
- 保留带参数快捷方式和管理员启动语义；前台激活失败时提示并闪烁任务栏，避免重复启动。
- 自动更新来源切换到 elias-chen/frun。
- 更新中英文 README、使用说明、构建与发布流程；提供便携 ZIP、单文件和 SHA-256。

First stable release of this fork, based on upstream 0.2.1: cursor-position middle-button hold, persistent pins, configurable delay, manual and per-app wake pause, existing-window activation, and fork-specific updates. Includes bilingual READMEs and verified portable release assets.

## 上游 / Upstream

此前版本历史见 [Nongfsq/frun Releases](https://github.com/Nongfsq/frun/releases)。
