## Release Process — Branch Model

### Branches

- `master` — release-only. Never commit development work directly here. Only receives merges from `dev` when shipping a release.
- `dev` — active development branch. All feature branches merge here.

### Versioning

Versions follow `MAJOR.MINOR.PATCH` with an optional numeric-only pre-release suffix (e.g. `2.4.0-1`, `2.4.0-2`). The suffix must be numeric-only because the MSI bundler rejects non-numeric pre-release identifiers.

### Release Candidates

Tag any version containing `-` from the `dev` branch. The release workflow detects the `-` and marks the GitHub release as a pre-release, which means:

- The installer is available under **GitHub → Releases** (with a "Pre-release" badge)
- Existing users are **not** notified by the auto-updater (the updater endpoint only serves `releases/latest`)
- The website is **not** updated

RC builds are triggered manually via `workflow_dispatch` on the `release.yml` workflow targeting `dev`.
