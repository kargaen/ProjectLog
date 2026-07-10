# EPIC-001: Shard ARCHITECTURE.md into constitution/ and description/

**Status:** closed
**Created:** 2026-07-10
**Architecture baseline:** 3396405 (pre-shard revision of ARCHITECTURE.md)

---

## 1. BDD — User Flows

Written retroactively: this epic documents work already shipped in commit `7ec61da` on
`dev`, at the user's request, so the decision has a durable record instead of living only in
chat.

### Flow 1: Selective reading

```gherkin
Given a contributor or agent needs only the import graph rule
When they open ARCHITECTURE.md
Then they find a one-line index entry pointing to `constitution/06-import-graph-rule.md`
And they read that one file instead of scanning the full 866-line document
```

### Flow 2: Write policy enforced by file location

```gherkin
Given an agent has just shipped a slice that changes the repo's structure
When it documents that change
Then it writes to a file under `architecture/description/`
And it never edits a file under `architecture/constitution/` without explicit human review
```

**Out of scope for this epic:**
- CODEOWNERS enforcement of the constitution/description write boundary. The directory split
  makes that enforceable, but no CODEOWNERS rule was added — the boundary is currently a
  convention (stated in `ARCHITECTURE.md`'s Write Policy), not a mechanically enforced one.
- Renumbering or cross-reference preservation. The original document had no section numbers
  and no internal cross-references, so this epic assigned fresh sequential numbers per class
  rather than preserving anything.

---

## 2. Function Call Signatures

*(not applicable — this epic restructures documentation, not code)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Authority | Use when | Example |
|---|---|---|
| **Legacy application output** | Splitting an existing document — the shard files must reproduce the original prose exactly | Pre-shard `ARCHITECTURE.md` (commit `3396405`) |

The pre-shard `ARCHITECTURE.md` is the legacy output. A shard file is not correct because it
reads well; it is correct because concatenating it back reproduces the original section's
prose byte-for-byte.

### Test map

| Flow | Function call | Authority | Fixture | Tolerance |
|---|---|---|---|---|
| 1, 2 | Reconstruction diff: each `architecture/**/*.md` section vs. the matching `## <Title>` block in the pre-shard file | Pre-shard `ARCHITECTURE.md` | Backup taken before sharding (`ARCHITECTURE.md` at commit `3396405`) | Exact match — zero prose differences allowed; only the trailing `---` separator line may differ, since that structural marker is superseded by file boundaries |

Two real mismatches surfaced during this diff (markdown table separator rows with a
miscounted dash, and a one-space indentation slip in the directory tree) — both were prose
differences, both were corrected, both re-verified before commit.

### What is deliberately not tested

- Visual formatting of the rendered index table — only content correctness was checked.
- Whether `architecture-md-maintenance`'s Change History convention is being followed yet —
  no Description section has been amended since the shard, so there is nothing to log.

---

## 4. Checklist

```md
[x] 1. Classify every section of `ARCHITECTURE.md` into constitution/description, confirmed with the user — done when the user replied "I want to move to a folder based architecture ... begin building it out."
[x] 2. Write `architecture/constitution/01..13-*.md` with prose moved verbatim from the matching sections — done when each file's content reconstruction-diffed clean against the backup.
[x] 3. Write `architecture/description/01..11-*.md` with prose moved verbatim from the matching sections — done when each file's content reconstruction-diffed clean against the backup.
[x] 4. Split the `Release Process` section across both classes (`constitution/13-release-branch-model.md` for branch roles/versioning constraint/RC definition, `description/12-release-shipping-procedure.md` for npm mechanics/CHANGELOG lifecycle/shipping steps) — done when the two files' concatenated content matched the original `Release Process` section exactly.
[x] 5. Rewrite root `ARCHITECTURE.md` as an index (Mission, Write Policy, Index table, Summary) — done when the index listed all 25 shard files with one description line each.
[x] 6. Fix the two real prose mismatches found by the reconstruction diff (table separator dash counts, directory-tree indentation) — done when the diff script reported zero non-separator differences.
[x] 7. Commit and push to `dev` — done when `7ec61da` landed on `origin/dev`.
```

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections: <list>
- [x] **Requires a Constitution change** — resolved before implementation: the user confirmed the constitution/description classification of every section (see checklist item 1) before any file was written. Not a blocker; recorded here because the write-policy split is itself a Constitution-level decision.

### North star deviation

North star: *"Every decision in this document exists to protect [minimal friction to log a
moment, and unshakeable trust in what gets reconstructed from it later]. An epic that trades
either away ... erodes the north star and must say so plainly."*

**No.** This epic only restructures documentation. It changes nothing a user of the app can
observe — no logging flow, no timesheet reconstruction path, no persisted format. It changes
how agents and contributors read and amend the *architecture document*, not the product.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Should a CODEOWNERS rule enforce that only humans can approve changes under `architecture/constitution/`? | No — the boundary is currently convention-only, stated in the Write Policy header | Before the first agent-proposed constitution edit is attempted |
| Q2 | `architecture-md-maintenance`'s folder-tree regeneration step depends on `scripts/tree.py`, which this repo does not yet have | Yes — blocks any future automated refresh of `description/04-directory-tree.md`; manual edits remain possible | Before the next directory-structure change needs documenting |

### New capability

None — this epic restructures existing documentation only; no product-facing capability
changed.
