## Release Process — Shipping Procedure

### Release candidates (automatic)

Every push to `dev` (except skills-only pushes — paths under `.claude/skills/**` are ignored) triggers `release-candidate.yml`. Its `check` job first runs the already-released-version guard (EPIC-009): it strips the pre-release suffix from `package.json`'s version and, if a stable release `v<base>` is already published, fails with an `::error::` naming the version and the fixing command (`npm run bump -- patch`), before any build work and leaving the current `rc` untouched. Past the guard it runs `npm run build` and the Rust tests, then publishes a single rolling pre-release under the `rc` tag with the Windows installer, built from the latest `dev` commit. The `rc` release and tag are unversioned — deleted and recreated on each run, so there is only ever one release candidate live — but the installer asset filenames carry `package.json`'s version. A newer `dev` push cancels an in-flight build. Existing users are not notified (the updater serves only `releases/latest`), and the website is not updated.

Producing an RC needs no manual step per push; the one manual step per release cycle is the post-release version bump the guard enforces.

### Versioning mechanics

To bump the version from the repo root:

```
npm version <new-version>
```

This updates `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock` atomically in a single commit, then creates a git tag.

For pre-release bumps between stable releases (EPIC-009), use instead:

```
npm run bump -- <patch|minor|major|rc> [--dry-run] [--from <version>]
```

`scripts/bump.mjs` (pinned by `scripts/bump.test.mjs`): `patch`/`minor`/`major` bump the base version and append `-1` (e.g. `2.4.0` → `2.4.1-1`); `rc` increments the numeric pre-release suffix (e.g. `2.4.1-1` → `2.4.1-2`) and errors when there is none. It applies the version via `sync-version.mjs` and commits the four version files — it never creates a git tag and never pushes. `--dry-run` prints the computed version and changes nothing.

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
