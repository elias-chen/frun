<p align="center">
  <img src="docs/media/logo.png" width="112" alt="frun">
</p>

<h1 align="center">frun</h1>

<p align="center">
  <b>Hold the middle mouse button. Release. Click an app.</b><br>
  A small Windows launcher with pinned apps and keyboard search.
</p>

<p align="center">
  <a href="README.zh-CN.md">中文</a> ·
  <a href="https://github.com/elias-chen/frun/releases/latest">Download</a> ·
  <a href="docs/usage.md">Usage guide (中文)</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="LICENSE">MIT License</a>
</p>

This is [elias-chen/frun](https://github.com/elias-chen/frun), a fork of [Nongfsq/frun](https://github.com/Nongfsq/frun). It adds a mouse launcher workflow while retaining the upstream program search, pinyin matching, path completion, and lightweight native Windows interface. No subscription is required.

## What's new in 0.3.0

- **Launch at the cursor.** Hold the middle mouse button for 400 ms to show the panel, release it, then left-click an app. The panel stays open after release. Its search field appears near the trigger position, clamped to the monitor's work area.
- **Adjust the hold time.** Settings → Wake offers a 200–1000 ms slider in 50 ms steps. Short clicks are replayed; moving more than 8 screen pixels before the threshold hands the middle-button drag back to the application.
- **Pin your apps.** Right-click a search result or press Ctrl+P to pin it. An empty query shows a four-column grid; typing returns to search. Pins and their order survive restarts.
- **Reuse existing windows.** Ordinary launches first try to restore and activate an existing app window, preferring the most recently used one. Shortcuts with arguments and administrator launches retain their original behavior. Use “New instance” in the context menu to bypass window reuse.
- **Pause when needed.** Pause or resume wake gestures and the global keyboard shortcut from Settings or the tray. Add executable names to the foreground-app exclusion list to pause automatically in games or other apps; switching away restores wake.

Manual pause lasts until you resume or restart frun. The exclusion list and hold duration are saved. Input replay and foreground activation depend on Windows permissions and each application's input handling; see the [usage guide](docs/usage.md) for details.

## Download and run

Requires **Windows 10 or 11, x64**. Download from [this fork's Releases](https://github.com/elias-chen/frun/releases/latest):

| Asset | Use |
| --- | --- |
| frun-v0.3.0-windows-x64.zip | Portable package with the executable, usage guide, and licenses |
| frun.exe | Standalone executable or manual replacement |
| frun.exe.sha256 | SHA-256 checksum used by the built-in updater |

Extract the ZIP and run frun.exe. When replacing an older copy, quit it from the tray first. This fork shares the upstream configuration, cache, and single-instance identity; back up your configuration before switching between builds. If Quicker also uses the middle button, disable its middle-button wake gesture or exit it first.

Enable launch at startup in Settings if desired. Automatic updates check **elias-chen/frun** and verify the published SHA-256 before replacing the executable when the panel hides. An upstream build still checks upstream releases, so install this fork manually the first time.

## Controls

| Input | Action |
| --- | --- |
| Hold middle mouse button | Show or hide at the cursor; release keeps the panel open |
| Alt+Space | Show or hide; configurable in Settings → Wake |
| Left click / Enter | Open the selected app, reusing an existing window when eligible |
| Ctrl+P / context menu | Pin or unpin an app |
| PageUp / PageDown / wheel | Change pinned-app pages |
| Alt+1 … 8 | Open the corresponding search result |
| → / ← | Expand or collapse other executables in a program folder |
| Tab | Complete a path, or move to the next result |
| Ctrl+Shift+Enter | Open as administrator |
| Ctrl+Enter | Open the containing folder |
| Ctrl+Delete / Ctrl+Z | Hide a program / undo |
| Ctrl+, | Settings |
| Esc | Go back or hide |

Search includes Start menu entries, Microsoft Store apps, and discovered program folders. Chinese names support pinyin and initials; paths support Tab completion. Without pins, an empty query keeps the original frequently used list. Themes follow Windows or your preference.

## Settings and data

Settings and usage counts are in `%APPDATA%\frun`; the index and icon cache are in `%LOCALAPPDATA%\frun`. Automatic update checks can be disabled in Settings. The foreground exclusion list matches executable filenames exactly, ignoring case; it does not send the list to a server.

For indexing problems, run `frun --doctor` or `frun --doctor "C:\path\to\program.exe"`. Review the report for personal paths before attaching it to an issue. See [CONTRIBUTING.md](CONTRIBUTING.md) for reporting and development.

To remove frun, disable launch at startup, quit from the tray, and delete the executable and the two data folders.

## Build and release

Install stable [Rust](https://rustup.rs) on Windows, then run:

```powershell
cargo check --locked --tests
cargo test --locked
cargo build --release --locked
```

The executable is at `target/release/frun.exe`. Windows CI checks, tests, and builds pull requests. The release workflow tests and builds on Windows, packages licenses, verifies uploaded assets, and publishes only after those steps succeed. See [CONTRIBUTING.md](CONTRIBUTING.md) for the version-bump release process.

## Credits and license

Original launcher by [Nongfsq](https://github.com/Nongfsq/frun); this fork adds the mouse workflow, pins, exclusions, pause controls, and window reuse. Source code is under [MIT](LICENSE); the original copyright notice is retained. The bundled Sarasa Gothic font subset uses SIL Open Font License 1.1: [font notice](assets/FONT-NOTICE.md) and [license](assets/FONT-LICENSE.txt).
