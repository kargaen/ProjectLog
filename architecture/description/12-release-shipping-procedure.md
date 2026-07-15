## Release Process — Shipping Procedure

### Release candidates (automatic)

Every push to `dev` (except skills-only pushes — paths under `.claude/skills/**` are ignored) triggers `release-candidate.yml`: it runs `npm run build` and the Rust tests, then publishes a single rolling pre-release under the `rc` tag with the Windows installer, built from the latest `dev` commit. It is unversioned — the same `rc` release and tag are deleted and recreated on each run, so there is only ever one release candidate live. A newer `dev` push cancels an in-flight build. Existing users are not notified (the updater serves only `releases/latest`), and the website is not updated.

Producing an RC needs no manual step; merging to `dev` is enough.

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

1. Update `CHANGELOG.md` — rename `## Unreleased` to the version + date
2. Run `npm version <version>` (no `-` suffix) so `package.json` carries a clean `MAJOR.MINOR.PATCH`
3. Merge `dev` into `master` and push `master`

The push to `master` triggers `release.yml`, which:

- refuses to run if the version still carries a pre-release suffix, and does nothing if that version is already released;
- builds the installer, tags `v<version>`, and publishes the stable release using `CHANGELOG.md` as the body;
- deletes the rolling `rc` pre-release, since the stable release supersedes it.

The website is deployed separately by `pages.yml` on the same `master` push.
