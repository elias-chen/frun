# Contributing to this fork

This repository is [elias-chen/frun](https://github.com/elias-chen/frun), based on [Nongfsq/frun](https://github.com/Nongfsq/frun). Report problems with this fork here rather than in the upstream project. Issues and pull requests may be written in English or 中文.

## Report a problem

Include the frun version, Windows version, steps to reproduce, and expected and actual behavior. For mouse or window issues, include the hold duration, pause/exclusion state, monitor scaling, and whether the affected app runs as administrator.

For missing or incorrectly indexed programs, run `frun --doctor` or `frun --doctor "C:\path\to\program.exe"`. The report contains local paths; review it before sharing.

## Develop

Use stable Rust on Windows 10 or 11, x64:

```powershell
cargo check --locked --tests
cargo test --locked
cargo build --release --locked
```

Keep changes focused and describe the user-visible behavior and validation in the pull request. Preserve the MIT notice and bundled font license. Changes to global input handling, foreground activation, or display placement also need Windows manual testing; the checklist is in [docs/usage.md](docs/usage.md).

## Release

1. Bump the package version in both Cargo.toml and the root frun entry in Cargo.lock.
2. Update both READMEs and CHANGELOG.md; add release notes at `docs/releases/v<version>.md`.
3. Merge after Windows PR checks pass. A push to main that changes Cargo.toml starts the release workflow. Dependency-only Cargo.toml changes also require a version bump for that workflow.
4. The workflow validates the version, tests and builds the x64 executable, optionally signs it when SignPath is configured, and creates a draft. It uploads the standalone executable, SHA-256, licenses, and portable ZIP, downloads them again for verification, then publishes the release as latest.

A pushed matching `v<version>` tag or a manual run of the release workflow on main can also start the same process. Tags must point at the exact source commit, and each version needs its own notes file. Runs are serialized. A rerun can finish an existing draft; an already published release at the same commit is left intact. A version tag pointing at a different commit fails instead of replacing published assets.

The built-in updater depends on assets named exactly `frun.exe` and `frun.exe.sha256`. Do not rename them or publish a release without them. No signing credentials are required for an unsigned build.
