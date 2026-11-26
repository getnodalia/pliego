# Nodalia migration: Pliego

Category: secondary app.
Status: NDL interface and first release (0.1.0), as of 2026-11-18. Evidence: pliego #32 (NDL), #33 (DMG window) and the 0.1.0 release pull request.

## Current state

Pure-Rust page-layout app (egui), MIT OR Apache-2.0. Cargo git dependencies come from getnodalia/foundation and nodalia-design. The gate, FreeBSD,
Nix, packaging lint, AGENTS.md and git-sources checks call getnodalia/foundation's reusable
workflows pinned by commit. SonarCloud was removed; CodeQL and Scorecard remain.

## Desired state

No workflow, dependency or document depends on storytold; Pliego has its own release workflow; the
interface follows Foundation's [NDL standard](https://github.com/getnodalia/foundation/blob/main/standards/ndl.md).

## Inherited dependencies still pending

| Dependency | Where | Replacement | Blocked on |
|---|---|---|---|
| `storytold/craft-common-actions` | `.github/workflows/windows-arm64.yml` | Foundation reusable workflows (release, refresh-contributors, windows-arm64) | writing and testing them |
| `storytold/craft-fonts` | `crates/fonts/build.rs`, `flake.nix`, `nix/package.nix` | getnodalia/foundation `vendor/craft-fonts/main` | repointing |
| `storytold/craft-libs` | `nix/package.nix`, `packaging/windows/README.md` | to decide | — |
| `storytold/indd-utils` | `Cargo.toml` | to decide | — |
| `storytold/pdfcraft` | `packaging/macos/package.sh` | to decide | — |
| `storytold/vectorcraft` | `packaging/macos/dmg/README.md`, `packaging/macos/package.sh` | to decide | — |
| `storytold/winprint.rs` | `Cargo.toml` | to decide | — |

## Names

Every name is Pliego's, internal ones included: crates (`pliego-*`), binaries (`pliego`,
`pliego-cli`), the app id `dev.danielmiguelt.Pliego`, settings folders and packages.
There is no upstream sync and no rename tooling. New
documents save as `.pliego`; `.designcraft` files keep opening, and the document signature and
media type inside the archive keep their old value so files stay readable both ways.

## Reusable Nodalia components

getnodalia/foundation (vendored crates, fonts, translations, reusable workflows);
getnodalia/nodalia-design (theme, icons).

## NDL

Done in #32: Nodalia Design tokens and islands, the creator suite's app bar with its workspace
selector and its Home, the hamburger menu with About, Preferences and Keyboard shortcuts, the
theme selector only in Preferences, NDL window controls on Linux only (native ones on macOS and
Windows), and the menus exported over D-Bus to the desktop's global menu on Linux. The app icon is
Benjamin Franklin's portrait; the macOS DMG opens its own designed window (#33).

## Risks and priorities

1. Releases still run the inherited release workflow.
2. Windows ARM64 packaging still runs the inherited workflow.

## Recommended order

1. A Foundation release workflow, tested with a dry run, then Pliego's release on it.
2. A Foundation Windows ARM64 workflow.

## What the first agent should do

1. Read AGENTS.md and this file.
2. Pick the first item of "Recommended order".

## Releases

Pliego releases through `release-auto.yml` like every Nodalia app (Foundation's `release-tag` and
`release-publish`, with `release-linux.yml`, `release-macos.yml` and `release-windows.yml`). The
packages are named `pliego-X.Y.Z-…`.
