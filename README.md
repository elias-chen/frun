<p align="center">
  <img src="docs/media/logo.png" width="112" alt="frun">
</p>

<h1 align="center">frun</h1>

<p align="center">
  <b>Alt + Space. A few letters. Enter.</b><br>
  A small, quiet program launcher for Windows.
</p>

<p align="center">
  <a href="README.zh-CN.md">中文</a>
  &nbsp;·&nbsp;
  <a href="#keys">Keys</a>
  &nbsp;·&nbsp;
  <a href="#get-it">Get it</a>
  &nbsp;·&nbsp;
  <a href="docs/usage.md">Guide (中文)</a>
  &nbsp;·&nbsp;
  <a href="CHANGELOG.md">Changelog</a>
  &nbsp;·&nbsp;
  <a href="LICENSE">MIT License</a>
</p>

<p align="center">
  <img src="docs/media/demo.en.gif" width="92%" alt="Type we, press Enter, and WeChat opens">
</p>

frun looks for programs and nothing else. It finds them wherever they live: the Start menu, the Microsoft Store, and the small tools you keep in folders like `D:\Tools` that never made it into the Start menu. Your documents stay out of the way.

## What it does

### Type a name, or its pinyin

`wx` finds 微信 (WeChat), `vsc` finds Visual Studio Code. Enter opens the top row; Alt + a number opens that row.

Opening an app first tries to bring its existing window forward, restoring it if minimized. If no suitable window exists, frun starts the app.

### Keep your everyday apps at hand

Right-click a search result, or press Ctrl + P, to pin an app to the home page. With the search field empty, your pins appear in a four-column grid; type a name and the search results return. Pins and their order are saved between runs.

Hold the middle mouse button to open the panel near your pointer, release it, then left-click an app. The panel stays open after release. The default hold time is 400 ms; you can adjust it or turn mouse wake off in Settings → Wake.

### One program, one row

A program folder often holds a dozen exes: the app, its updater, crash reporters, helpers. frun lists the program once and keeps the rest behind it. Press → to see them.

<p align="center">
  <img src="docs/media/one-row.en.jpg" width="80%" alt="VirtualBox shows as one row; the arrow key lists the other exes in its folder">
</p>

### Paths work too

Type `d:\so`, press Tab to complete, keep typing, press Enter, and the folder opens in Explorer.

<p align="center">
  <img src="docs/media/path.jpg" width="80%" alt="Typing a path with Tab completion">
</p>

### Right-click for the rest

Open as administrator, open the containing folder, hide a program you never want to see again (Ctrl + Z brings it back), or stop searching a folder. Choose *New instance* to bypass frun's existing-window reuse.

<p align="center">
  <img src="docs/media/menu.en.jpg" width="80%" alt="The right-click menu">
</p>

### Light and dark

It follows the Windows theme, or you can pick one. With more than one screen, it opens on the screen your mouse is on.

<p align="center">
  <img src="docs/media/light.en.jpg" width="80%" alt="Settings in the light theme">
</p>

### Open everything as administrator, if you want

Turn on *Open as administrator* in Settings → General, and every program you start from frun runs elevated. Windows still asks each time, as it should. frun itself never asks for administrator rights. Microsoft Store apps are left alone, because Windows does not let them run elevated this way.

<p align="center">
  <img src="docs/media/admin-switch.en.jpg" width="49%" alt="The Open as administrator switch">
  <img src="docs/media/admin-run.en.jpg" width="49%" alt="Windows PowerShell opened from frun, running as administrator">
</p>

### Nothing to set up

On first run frun reads the install records Windows keeps, works out which folders hold your programs, and adds them. When you install something new, it shows up on its own.

### Pause when you need to

Pause wake from Settings or the tray menu to release both the middle mouse button and the global keyboard shortcut. You can also add games or other apps to the exclusion list: wake pauses while one is in the foreground and resumes when you switch away.

Manual pause lasts until you resume or restart frun. More settings and application-specific behavior are covered in the [usage guide](docs/usage.md).

## Keys

| Key | What it does |
| --- | --- |
| Alt + Space | Show or hide (you can change it in Settings → Wake) |
| Hold middle mouse button | Show or hide near the pointer; release keeps a newly opened panel visible |
| Enter / left click | Open an app; reuse an existing window when eligible |
| Ctrl + P | Pin or unpin the selected app |
| PageUp / PageDown / wheel | Change pages in the pinned-app grid |
| Alt + 1 … 8 | Open that row |
| → | Show the other exes in a program's folder (← folds them back); accept a path completion |
| Tab | Accept a path completion; otherwise move to the next row |
| Ctrl + Shift + Enter | Open as administrator |
| Ctrl + Enter | Open the containing folder |
| Ctrl + Delete | Hide this program; Ctrl + Z to undo |
| Ctrl + , | Settings |
| Esc | Back, or hide |

## Small on purpose

One exe of about 2 MB, written in Rust. No installer, no runtime, no web view.

The figure below shows upstream's measurements of the original build with 1,603 programs indexed. This fork has not been benchmarked separately.

<p align="center">
  <img src="docs/media/stats.en.png" width="100%" alt="0.4 MB in memory once hidden, 0.25 ms per keystroke, 99.4% of folders skipped, 2 MB for the whole app">
</p>

## Get it

Download `frun.exe` from [Releases](https://github.com/elias-chen/frun/releases/latest), put it anywhere you like, and run it. It needs Windows 10 or 11, x64. A portable ZIP with a usage guide and licenses is also available.

Turn on *Launch at startup* in Settings if you want it there after a restart. New versions arrive on their own: frun checks this fork's releases, downloads updates in the background, verifies the published SHA-256, and switches over the next time it hides.

When switching from upstream, download this fork manually and quit the old copy from the tray first. Both versions share configuration and cache directories; you can back up `%APPDATA%\frun` before switching. Changes between versions are in [CHANGELOG.md](CHANGELOG.md).

The exe is not code-signed yet, so the first time you run it Windows SmartScreen may say it protected your PC. Choose *More info*, then *Run anyway*.

To build it yourself instead, install stable [Rust](https://rustup.rs) on Windows and run:

```powershell
cargo build --release --locked
```

To remove frun, turn off *Launch at startup*, quit it from the tray icon, and delete `frun.exe` along with the `%APPDATA%\frun` and `%LOCALAPPDATA%\frun` folders.

## Your data

Settings and usage counts live in `%APPDATA%\frun`; the index and icon cache live in `%LOCALAPPDATA%\frun` and can be deleted at any time. Searches, settings, and usage counts stay on your computer. Network access is used for version checks, at most once a day, and update downloads. You can turn automatic updates off in Settings.

If frun misses a program or shows one it shouldn't, run `frun --doctor`. It writes a report to `%LOCALAPPDATA%\frun\doctor.txt` and opens it. The report lists folder and program paths from your computer, so read it before you attach it to an issue. See [CONTRIBUTING.md](CONTRIBUTING.md) for reporting problems and contributing.

## License

This is a fork of [Nongfsq/frun](https://github.com/Nongfsq/frun). The demo and interface screenshots come from upstream; some settings differ in this fork.

MIT, see [LICENSE](LICENSE). The interface font is a subset of Sarasa Gothic under the SIL Open Font License 1.1, see [assets/FONT-NOTICE.md](assets/FONT-NOTICE.md). The Rust logo in the figure under *Small on purpose* is © the Rust Foundation, used under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) and recolored white; frun is not affiliated with the Rust project.
