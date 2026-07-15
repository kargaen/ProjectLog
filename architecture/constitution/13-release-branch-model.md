## Release Process — Branch Model

### Branches

- `master` — release-only. Never commit development work directly here. Only receives merges from `dev` when shipping a release. Pushing `master` triggers the stable release build.
- `dev` — active development branch. All feature branches merge here.

### Versioning

Versions follow `MAJOR.MINOR.PATCH` with an optional numeric-only pre-release suffix (e.g. `2.4.0-1`, `2.4.0-2`). The suffix must be numeric-only because the MSI bundler rejects non-numeric pre-release identifiers.

### Release Candidates

Release candidates are automatic and unversioned. Every push to `dev` (skills-only pushes excepted) builds and publishes a single rolling pre-release under the `rc` tag, which means:

- The installer is available under **GitHub → Releases** (with a "Pre-release" badge), always reflecting the latest `dev` commit
- Only one release candidate exists at a time — each build replaces the previous `rc`
- Existing users are **not** notified by the auto-updater (the updater endpoint only serves `releases/latest`)
- The website is **not** updated

There is no manual RC step and no RC version number; merging to `dev` produces the candidate. Shipping a stable release deletes the `rc` pre-release.
