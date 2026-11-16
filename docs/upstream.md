# Pliego upstream

| Field | Value |
|---|---|
| Upstream | `storytold/designcraft` |
| Last observed | `95abb2b03b6d510a72d687b2e5ee41ef3cbd7485` |
| Last evaluated | `95abb2b03b6d510a72d687b2e5ee41ef3cbd7485` |
| Last integrated | `95abb2b03b6d510a72d687b2e5ee41ef3cbd7485` merged on 2026-11-10 |
| Skipped | none in the baseline import |
| Adapted | every name: crates, binaries, app id, settings folders, the document extension |
| Conflicting | `README.md` (kept ours), `AGENTS.md` (upstream plus our quality contract) |

## How the sync works

Upstream's code arrives renamed: `scripts/rebrand.py` applies `.github/rebrand.toml` (DesignCraft
becomes Pliego, internal names included) to upstream's tree, and the weekly sync
(`.github/workflows/upstream-sync.yml`) records the result on the `upstream-pliego` branch. Each
commit there has the previous mirror commit and the upstream commit it renames as parents, and
names that commit in an `Upstream-Commit:` trailer. The sync merges the newest mirror commit
into a branch and opens a pull request; files listed in `.github/upstream-keep` keep this
repository's version.

The mirror starts at the renamed `95abb2b0`, merged into `main` (with this repository's tree)
by the pull request that renamed the app. `python3 scripts/rebrand.py check` fails when an old
name comes back; a line that must keep one says `rebrand: keep`.

## What stays as it was

Files saved before the rename keep opening, and files Pliego saves stay readable by older
versions:

- The document signature `"format": "designcraft"` and the archive's media type
  `application/vnd.designcraft+zip`. <!-- rebrand: keep -->
- New documents save as `.pliego`; `.designcraft` files open and save as before <!-- rebrand: keep -->
  (`pliego_format::OLDER_EXTENSIONS`).
