# EPIC-009: RC version discipline

**Status:** closed
**Created:** 2026-07-16
**Architecture baseline:** cd91662

Adds a fail-loud guard to the rolling-rc pipeline — a `dev` push whose version is already
published as a stable release must not silently rebuild the rc — and a one-command version
bump that unblocks it. The rolling rc model itself (one unversioned `rc` release, versioned
asset filenames) is unchanged; EPIC-007's version-targeted-RC design remains superseded.

---

## 1. BDD — User Flows

### Flow 1: Fail loud when the version was already released

```gherkin
Given a stable release for version X.Y.Z is published
And the repository's version is still X.Y.Z, or any X.Y.Z-N pre-release of it
When a commit is pushed to dev
Then the release-candidate build fails before any installer is built
And the failure message names the offending version and the exact command that fixes it
And the existing rc release, tag, and assets are left untouched
```

### Flow 2: One-command bump unblocks the pipeline

```gherkin
Given the release-candidate build failed because the version was already released
When the maintainer runs the patch (or minor) bump command and pushes the result
Then all version files carry the next patch (or minor) base with a numeric pre-release suffix (e.g. 2.4.1-1)
And the next dev push passes the guard
And the new rc's asset filenames carry the new version
```

### Flow 3: Rolling rc invariant (existing behaviour, protected here)

```gherkin
Given an rc pre-release exists
When a dev push passes the guard and the build succeeds
Then the previous rc release and its tag are deleted
And exactly one rc pre-release exists, built from the latest dev commit
And its asset filenames carry the current version from package.json
```

**Out of scope for this epic:**
- Branch protection on `master` — a GitHub repository setting the maintainer applies by
  hand; no code can be written for it here.
- A literal `-rc` text suffix (e.g. `2.4.1-rc.1`) — the MSI bundler rejects non-numeric
  pre-release identifiers; the numeric-only rule in `constitution/13` stands (same exclusion
  as EPIC-007).
- Versioned rc *tags* or retaining multiple rcs — the rolling single-`rc` model stands; only
  asset filenames carry the version.
- Any change to the stable release path (`release.yml`) or to `scripts/release.mjs`.
- Guarding against a *clean* unreleased version on dev (e.g. `2.4.1` with no suffix) — that
  is the legitimate final pre-ship state per `description/12` and must keep building.

---

## 2. Function Call Signatures

Only the bump command's CLI contract constrains other work (the guard is a workflow step
with no callers):

```
npm run bump -- <patch|minor|major|rc> [--dry-run]
```

- `patch|minor|major` — bump the base version, reset lower parts, append `-1`.
- `rc` — increment the numeric pre-release suffix; error if the current version has none.
- `--dry-run` — print the computed version and change nothing.
- Applies via the existing `scripts/sync-version.mjs` (package.json, package-lock.json,
  Cargo.toml, Cargo.lock) and commits; it never creates or pushes git tags.

---

## 3. TDD — Testing Strategy

The guard is CI shell logic: the repo has no CI unit-test harness, so (as in EPIC-007) its
authority is an observed workflow run plus GitHub Releases API state. The bump computation
is pure and is pinned by the case table below.

### Authority for correctness

| Flow | What is checked | Authority | Fixture |
|---|---|---|---|
| 1 | Guard step exits non-zero with an `::error::` naming the version and fix, before any build work; rc release untouched | Observed workflow run; `gh release view rc` unchanged | A dev push whose base version matches a published stable release |
| 2 | Bump computes the next version per the case table | Case table below (semver ordering + `constitution/13` numeric-only-suffix rule) | `node scripts/bump.mjs <type> --dry-run` over every row |
| 2 | Bumped push passes the guard and assets carry the new version | Observed workflow run; asset filenames in the rc release | The first dev push after a bump |
| 3 | Exactly one rc exists after a passing build | Existing behaviour; observed run (regression check only, no new code) | Any passing dev push |

### Case table (bump authority)

| Current version | Command | Result |
|---|---|---|
| `2.4.0` | `patch` | `2.4.1-1` |
| `2.4.0-4` | `patch` | `2.4.1-1` |
| `2.4.1-2` | `minor` | `2.5.0-1` |
| `2.4.0-4` | `major` | `3.0.0-1` |
| `2.4.1-2` | `rc` | `2.4.1-3` |
| `2.4.0` | `rc` | error: no pre-release suffix to increment |

Guard rule: strip the pre-release suffix from `package.json`'s version; if a published
release `v<base>` exists, fail. (Covers both the forgotten bump after shipping and a
lingering `X.Y.Z-N` after `X.Y.Z` shipped; a clean unreleased base passes by design.)

### What is deliberately not tested

- The Windows installer's runtime correctness — tauri-action's concern, unchanged.
- Exact error wording beyond naming the version and the fixing command.
- Concurrent dev pushes — the existing `concurrency` group already serialises rc builds.
- `scripts/sync-version.mjs` internals — pre-existing, unchanged.

---

## 4. Checklist

```md
[x] 1. Add `scripts/bump.mjs` — done when `node scripts/bump.mjs <type> --dry-run` reproduces every row of the §3 case table, including the error row
[x] 2. Add the `bump` script entry to `package.json` — done when `npm run bump -- rc --dry-run` prints `2.4.0-5` (current version `2.4.0-4`)
[x] 3. Add the already-released-version guard as the first step of the `check` job in `.github/workflows/release-candidate.yml` — done when the step's script, run locally with a stub `gh` that reports the base as released, exits non-zero with an `::error::` line, and exits zero when it does not; final confirmation on the next real dev push
```

*(No `constitution/13` edit appears here: that is a human decision, tracked in §5.
`description/12` is amended by `epic-closeout` after the guard ships, not here.)*

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections: `description/12-release-shipping-procedure.md` (via `epic-closeout`, after shipping)
- [x] **Requires a Constitution change — BLOCKS this epic until resolved.**
  **Resolved 2026-07-16:** maintainer approved the paste-text below verbatim ("I approve the
  change to constitution"). Applying it to `constitution/13` is a maintainer action (agents
  never edit constitution files); implementation proceeds on the approved wording.

`constitution/13-release-branch-model.md` currently says *"There is no manual RC step and no
RC version number; merging to `dev` produces the candidate."* This epic keeps the per-push
automation but introduces one required manual step per release cycle: after a stable release
ships, dev builds fail until the version is bumped. Proposed paste-text for the maintainer
to apply (replacing the final paragraph of the Release Candidates section):

> Producing an rc needs no manual step per push, and the rc release itself carries no
> version — but asset filenames do, taken from `package.json`. A version whose base is
> already published as a stable release never rebuilds as an rc: the build fails loudly
> until the version is bumped. After shipping, the first dev change therefore starts with
> `npm run bump -- patch` (or `minor`/`major`), producing the next base with a numeric-only
> pre-release suffix (e.g. `2.4.1-1`). Shipping a stable release deletes the `rc`
> pre-release.

### North star deviation

North star: *"minimal friction to log a moment, and unshakeable trust in what gets
reconstructed from it later."*

**No.** Developer-facing release infrastructure; nothing a user of the app observes changes.
It removes an ambiguity (rc assets silently carrying an already-shipped version), which is
in the spirit of the trust goal, not a trade against it.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Should `npm run bump` commit only (proposed default — push stays a deliberate act), or also push so the rc build triggers immediately? | nothing — item 1 implements the default unless overruled | epic-review — **default stands (2026-07-16): commit only, no push** |
| Q2 | Is a red workflow run loud enough (GitHub emails the pusher on failure by default), or should the guard also open/annotate something? Proposed: red run is enough. | nothing | epic-review — **default stands (2026-07-16): red run is enough** |

### New capability

None new to the product. A developer-facing guard and bump command; the rolling rc model
and stable release path are untouched.
