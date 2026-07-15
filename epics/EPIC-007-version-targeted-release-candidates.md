# EPIC-007: Version-targeted release candidates

**Status:** superseded
**Created:** 2026-07-13
**Architecture baseline:** bb2c76b

> **Superseded (2026-07-13, never implemented).** The maintainer chose the simpler rolling
> model instead of version-targeted RCs: `dev` push → one rolling unversioned `rc` pre-release
> (`release-candidate.yml`), `master` push → stable tagged release (`release.yml`). See
> `architecture/description/12-release-shipping-procedure.md`. This epic's Option-A design and
> its open questions are moot; kept for the record, not for implementation.

Reworks the RC path in `.github/workflows/release.yml` so every release candidate is a
candidate *for a declared next version*, fails loudly when no such version is declared, and
is cleaned up when its stable version ships. Option A of the pre-release review; the literal
`-rc` text label and a rolling "nightly" channel are both out of scope (see §1).

---

## 1. BDD — User Flows

### Flow 1: RC for a declared next version

```gherkin
Given the maintainer has declared a next target version X.Y.Z greater than the latest published stable release
When they trigger a release-candidate build from the development branch
Then a pre-release vX.Y.Z-N is published (N is the release-candidate counter)
And it is marked as a pre-release
And the website is not deployed
And the release notes mark it as a candidate for testing only
```

### Flow 2: Fail loud when no valid next version is declared

```gherkin
Given no next target version is declared, or the declared version is not greater than the latest published stable release
When a release-candidate build is triggered
Then the build fails immediately with an error message naming exactly what is wrong
And no release, tag, or asset is published
```

### Flow 3: Shipping the stable version removes its candidates

```gherkin
Given one or more pre-releases vX.Y.Z-N exist
When the stable release X.Y.Z is published
Then every vX.Y.Z-* pre-release and its tag are deleted
```

**Out of scope for this epic:**
- The literal `-rc` text suffix (e.g. `2.4.0-rc.1`). The WiX MSI target rejects non-numeric
  pre-release identifiers; the numeric-suffix rule in `constitution/13` stands. Changing this
  would mean dropping MSI for NSIS — a separate packaging (dependency) decision.
- A rolling / "nightly" versionless RC channel. A missing declared version **fails** (Flow 2);
  it never falls back to a versionless build. A nightly channel, if ever wanted, is its own
  trigger and its own epic.
- Auto-triggering an RC on every dev push (see Open Question Q4 — the trigger mechanism is
  unresolved; the flows do not mandate one).
- The stable-release build itself (branch model, `npm version`, CHANGELOG body). Only the RC
  lifecycle and the stable path's *RC cleanup* are in scope here.

---

## 2. Function Call Signatures

*(deferred to revision 2 — this epic is workflow YAML + shell steps, not a typed contract another layer depends on. The one contract that matters — how the target version is declared and read — is an unresolved Open Question, §5 Q1, and must be settled before signatures mean anything.)*

---

## 3. TDD — Testing Strategy

This is CI workflow + shell logic. The repo has no CI unit-test harness, and GitHub Actions
behavior can only be pinned by running it. Authorities are therefore observation of real
workflow runs against throwaway inputs, plus the GitHub Releases API state after each run.

### Authority for correctness

| Flow | What is checked | Authority | Fixture |
|---|---|---|---|
| 1 | A dispatched RC for a declared version publishes exactly one `vX.Y.Z-N` pre-release, prerelease flag set, website job skipped | Observed workflow run + GitHub Releases API (`gh release view vX.Y.Z-N --json isPrerelease,tagName`); website job shows `skipped` | A scratch declared version above the latest stable, dispatched from a scratch branch |
| 2 | Missing / non-greater version aborts in the `validate` job with a non-zero exit and an `::error::` line; no release created afterwards | Observed run: `validate` concludes `failure`; `gh release list` shows no new tag | Two dispatches: (a) no version declared, (b) a version ≤ latest stable |
| 3 | After a stable publish, no `vX.Y.Z-*` pre-release remains | Observed run + `gh release list` filtered to `vX.Y.Z-` returns empty | A base with ≥1 existing RC, then a stable release of that base |

Counter derivation (N) and the declaration mechanism are **authority TBD** until §5 Q1–Q3 are
decided — those checklist items cannot start until then (§4 marks them).

### What is deliberately not tested

- The Windows installer's runtime correctness — that is the tauri-action's concern, unchanged
  by this epic.
- The exact wording of the `::error::` messages beyond naming the failing condition.
- Any assertion that the numeric suffix produces a valid MSI — that constraint is pre-existing
  and already enforced by the bundler; this epic does not relax it.
- Concurrency / two RCs dispatched at once — single-dispatch behavior only, unless Q4 makes
  auto-trigger real.

---

## 4. Checklist

Ordered by dependency. Items 1–3 are **blocked** on the Open Questions and must not start
until those are answered at review (they are listed so the plan is visible, not so they are
begun).

```md
[ ] 1. (blocked on Q1) Add the declared-next-version source and read it in the `validate` job of `.github/workflows/release.yml` — done when a run with a valid declared version exposes it as the resolved build version
[ ] 2. Add the fail-loud guard to the `validate` job of `.github/workflows/release.yml`: abort with `::error::` + non-zero exit when no version is declared or it is not greater than the latest published stable — done when Flow 2's two fixture dispatches both conclude `failure` with no release created
[ ] 3. (blocked on Q2/Q3) Replace the `-1` normalization with the agreed RC-counter derivation in `.github/workflows/release.yml` — done when two successive RC dispatches of the same base publish distinct, increasing N per Flow 1
[ ] 4. Add the RC-cleanup step to the stable path in `.github/workflows/release.yml` (delete `vX.Y.Z-*` pre-releases when a stable X.Y.Z publishes) — done when Flow 3's fixture leaves no matching pre-release
[ ] 5. Update `.github/workflows/release.yml` `workflow_dispatch` input description to match the new declared-version contract — done when the input text no longer claims "normalized to -1"
```

*(No `constitution/13` edit appears here: that is a human decision, tracked in §5, and lands
outside this checklist. `description/12` is amended by `epic-closeout` after the workflow
ships, not here.)*

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections: `description/12-release-shipping-procedure.md` (via `epic-closeout`, after the workflow ships)
- [x] **Requires a Constitution change — BLOCKS this epic until resolved.**

`constitution/13-release-branch-model.md` currently defines the RC contract as "tag any
version containing `-` from dev; RC builds triggered via `workflow_dispatch`… suffix
normalized to `-1`." Option A changes that contract to: RCs target a **declared next base
version**, carry an **incrementing** numeric counter, **fail loud** when undeclared, and are
**deleted when their stable version ships**. The numeric-only-suffix rule is unchanged. This
is Constitution-class wording and a human must approve it before implementation — I will hand
over paste-text for `constitution/13`; I do not edit constitution files. Until that lands,
`epic-review` should hold this epic blocked.

### North star deviation

North star: *"a frictionless time-tracking companion… minimal friction to log a moment, and
unshakeable trust in what gets reconstructed."*

**No.** This is developer-facing release infrastructure; it touches no logging, timesheet, or
persistence behavior. If anything it *raises* release safety — no silent or ambiguous RC
artifacts, and no stale candidates lingering after a stable ships — but it changes nothing a
user of the app observes.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | How is the target next version declared? Options: bump `package.json` base, a dedicated `NEXT_VERSION` file, or a required `workflow_dispatch` input. Trade: `package.json` is one source of truth but couples to the file; a dispatch input is ephemeral, not recorded in git; a `NEXT_VERSION` file is explicit and reviewable but is a new artifact. | Item 1 | epic-review |
| Q2 | How is counter N derived, given RC cleanup deletes old tags so the last N cannot be read back? Options: GitHub run number, count of dev commits since the base was declared, or stop deleting per-N tags. | Item 3 | epic-review |
| Q3 | Keep all RCs of a base visible (`-1`, `-2`, `-3`) or only the latest? Keeping all clutters Releases; keeping only latest loses the counter source, so this must be resolved together with Q2. | Item 3 | epic-review |
| Q4 | Does a dev push auto-trigger an RC, or does it stay manual (`workflow_dispatch` / tag)? Flows 1–3 hold under either; auto-trigger widens scope and testing. | — (flows hold either way) | epic-review |

### New capability

None new to the product. A developer-facing change to how release candidates are versioned
and retired — the north star already assumes a shippable, testable app; this only makes the
candidate pipeline intentional.
