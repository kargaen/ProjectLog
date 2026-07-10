## Release Process — Shipping Procedure

### Versioning mechanics

To bump the version from the repo root:

```
npm version <new-version>
```

This updates `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock` atomically in a single commit, then creates a git tag.

### CHANGELOG.md

`CHANGELOG.md` in the repo root accumulates human-written release notes during development. Entries should be added as features and fixes are merged — not retroactively at release time.

**Format** (append new entries at the top, under a `## Unreleased` heading):

```md
## Unreleased

- Fixed taskbar button not hiding when always-on-top is enabled
- Rounding is now applied consistently on both preview and export
```

On release:

1. Rename `## Unreleased` to the release version and date (e.g. `## 2.4.0 — 2026-06-18`)
2. The release workflow reads `CHANGELOG.md` and uses its content as the GitHub release body
3. After the release is published, clear `CHANGELOG.md` back to an empty `## Unreleased` heading so new entries can accumulate for the next release

### Shipping a Release

1. Merge `dev` into `master`
2. Update `CHANGELOG.md` — rename `## Unreleased` to the version + date
3. Run `npm version <version>` on `master` (no `-` suffix)
4. Push `master` — the tag push triggers `release.yml`, which builds the installer, publishes the release using `CHANGELOG.md` as the body, and deploys the website
