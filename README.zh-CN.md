<p align="center">
  <img src="docs/media/logo.png" width="112" alt="frun">
</p>

<h1 align="center">frun</h1>

<p align="center">
  <b>长按中键呼出，松开，再左键选择软件。</b><br>
  支持固定软件、拼音搜索和已有窗口唤醒的 Windows 启动器。
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="https://github.com/elias-chen/frun/releases/latest">下载</a> ·
  <a href="docs/usage.md">使用说明</a> ·
  <a href="CHANGELOG.md">更新日志</a> ·
  <a href="LICENSE">MIT 许可</a>
</p>

这是 [elias-chen/frun](https://github.com/elias-chen/frun)，基于 [Nongfsq/frun](https://github.com/Nongfsq/frun) 改造的分支。在原有程序搜索、拼音匹配、路径补全和原生 Windows 界面的基础上，增加鼠标启动面板，无需会员订阅。

## 0.3.0 新增功能

- **在鼠标处呼出。** 默认长按中键 400 毫秒显示面板，松开后面板保留，再用左键点击软件。搜索框靠近触发位置，屏幕边缘自动向工作区内调整，支持多显示器与 DPI 缩放。
- **长按时长可调。** 设置 → 唤醒中提供 200～1000 毫秒滑块，步长 50 毫秒。短按补发原始点击；达到阈值前移动超过 8 个屏幕像素，会交还中键拖动。
- **固定常用软件。** 搜索后右键或 Ctrl+P 固定到首页。空搜索框显示四列图标，输入即回到搜索；固定目标和顺序重启后保留。
- **优先唤醒已有窗口。** 普通启动先尝试恢复并切换到已有窗口，多窗口时优先最近使用的窗口。带参数快捷方式、管理员启动保留原有行为；右键“新开实例”可跳过窗口复用。
- **暂停与应用排除。** 在设置或托盘暂停唤醒，同时释放中键和全局快捷键。将游戏等程序加入禁用名单后，仅在它处于前台时自动暂停，切出后恢复。

手动暂停只对本次运行有效，退出重开后恢复；长按时长和禁用名单会保存。短按补发、特殊输入和窗口切换受 Windows 权限与应用行为影响，详见[使用说明](docs/usage.md)。

## 下载与运行

支持 **Windows 10 / 11，x64**。请从[本分支的 Releases](https://github.com/elias-chen/frun/releases/latest) 下载：

| 文件 | 用途 |
| --- | --- |
| frun-v0.3.0-windows-x64.zip | 便携包，含程序、使用说明与许可证 |
| frun.exe | 单文件运行，或手动替换旧版本 |
| frun.exe.sha256 | SHA-256 校验文件，供自动更新使用 |

解压便携包后运行 frun.exe 即可。替换旧版本前，先在托盘右键退出旧进程。本分支与原版共用配置、缓存和单实例标识，切换版本前可备份配置。若 Quicker 同时占用中键，请先关闭它的中键唤醒或退出 Quicker。

可在设置中开启开机自启。自动更新检查 **elias-chen/frun** 的发布版本，用 SHA-256 核对后，在面板收起时替换程序。原版仍检查原仓库，首次切换到本分支需要手动下载。

## 操作

| 操作 | 作用 |
| --- | --- |
| 长按鼠标中键 | 在鼠标位置呼出或收起；松开后保留面板 |
| Alt+Space | 呼出或收起，可在设置 → 唤醒修改 |
| 左键 / Enter | 打开软件，符合条件时复用已有窗口 |
| Ctrl+P / 右键菜单 | 固定或取消固定 |
| PageUp / PageDown / 滚轮 | 固定软件翻页 |
| Alt+1 … 8 | 打开对应搜索结果 |
| → / ← | 展开或收起程序目录中的其他 exe |
| Tab | 路径补全，或移到下一项 |
| Ctrl+Shift+Enter | 以管理员身份打开 |
| Ctrl+Enter | 打开所在文件夹 |
| Ctrl+Delete / Ctrl+Z | 隐藏软件 / 撤销 |
| Ctrl+, | 设置 |
| Esc | 返回或收起 |

保留开始菜单、Microsoft Store 应用和程序目录索引，支持中文、拼音首字母和路径补全。没有固定软件时，空搜索框继续显示原有常用软件列表；主题可跟随系统或手动选择。

## 配置与数据

设置和使用次数位于 `%APPDATA%\frun`，索引和图标缓存位于 `%LOCALAPPDATA%\frun`。自动更新可在设置中关闭。禁用应用按 exe 文件名精确匹配、忽略大小写，名单保存在本地。

遇到索引问题，可运行 `frun --doctor` 或 `frun --doctor "C:\path\to\program.exe"`。报告包含本机路径，提交 issue 前请检查。反馈和开发说明见 [CONTRIBUTING.md](CONTRIBUTING.md)。

卸载时，关闭开机自启，从托盘退出，再删除程序和上述两个数据目录。

## 自行构建与发布

在 Windows 安装稳定版 [Rust](https://rustup.rs)，然后运行：

```powershell
cargo check --locked --tests
cargo test --locked
cargo build --release --locked
```

产物为 `target/release/frun.exe`。PR 会执行 Windows 检查、测试和构建；正式发布会在测试和构建通过后打包许可证，回读并核对上传资产，最后公开 Release。版本更新流程见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 来源与许可

原始项目为 [Nongfsq/frun](https://github.com/Nongfsq/frun)，本分支增加鼠标交互、固定软件、暂停、应用排除和窗口复用。代码采用 [MIT](LICENSE)，保留原作者版权声明。界面使用更纱黑体子集，遵循 SIL Open Font License 1.1，见[字体说明](assets/FONT-NOTICE.md)和[字体许可证](assets/FONT-LICENSE.txt)。
