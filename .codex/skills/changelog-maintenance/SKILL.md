---
name: changelog-maintenance
description: Use this skill whenever CHANGELOG.md is to be written — when an epic reaches Status closed, when a dependency-change run or direct slice lands something user-visible, or when a release candidate is promoted to a release. Appends one product-language entry per finished epic under [Unreleased] (never per slice, never per prompt), and at release renames [Unreleased] to the version heading and opens a fresh empty buffer. Do not use mid-epic, and never clear or rewrite released sections — the changelog is append-only history.
---

# Maintaining CHANGELOG.md

## Input

$ARGUMENTS

**Required:** what finished — a closed epic, a landed `dependency-change` run or direct
slice, or the version being released.
**If missing:** ask. Do not scan the conversation for something to log.

**Not waivable:** one entry per epic, written at close — not per slice, not per prompt. An
entry-per-prompt changelog is incoherent by construction.

## Format

Keep a Changelog convention. `[Unreleased]` at the top is the only section agents append to.
Released sections are history: never edited, never deleted, never "cleared".

```md
# Changelog

## [Unreleased]

### Added
- <entry> (EPIC-<NNN>)

## [1.4.0] — 2026-06-30
...
```

Categories: `Added`, `Changed`, `Fixed`, `Removed`, `Deprecated`, `Security`. Use only the
ones needed.

## Writing an entry

Triggered by `epic-closeout` when Status flips to `closed` — one entry for the whole epic:

- Product language, not code language: what a user of the product can now do, or what
  behaves differently. The epic's §1 flows are the source; the checklist is not.
- One to three lines. Cite the epic id.
- A `dependency-change` run or direct slice gets an entry only if user-visible (usually
  `Fixed` or `Security`); internal swaps stay out of the changelog — the Change History in
  the architecture document already records them.
- If `CHANGELOG.md` does not exist, create it with an `[Unreleased]` section and this first
  entry. That is the entire bootstrap.

## Rolling a release

Triggered by `branch-lifecycle` at promotion (gate 2 passed):

1. Rename `[Unreleased]` to `[<version>] — <date>`.
2. Insert a fresh empty `[Unreleased]` above it.
3. This edit is part of the release commit that gets tagged — the tag's notes are this
   section.

Release candidates get **no** section and no rename. An RC is built with `[Unreleased]`
intact — that buffer is its release notes while testing. Discarded RCs therefore leave no
trace in the changelog.

If `[Unreleased]` is empty at promotion, stop and say so — releasing nothing describable is
usually a sign closeout entries were skipped.

## Report

Follows the repository's reply rules: outcome line (the entry or the rolled heading as
evidence), next line.
