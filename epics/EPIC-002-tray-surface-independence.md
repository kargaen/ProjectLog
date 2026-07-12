# EPIC-002: Tray-surface independence

**Status:** draft
**Created:** 2026-07-12
**Architecture baseline:** 299b448

Source: migrated from `DEVNOTES.md` (deleted in the same commit). Original text quoted per flow.

---

## 1. BDD — User Flows

### Flow 1: Add / Quick-add a project from the tray without the QuickPanel

> DEVNOTES.md: "Tray add/quick project should just be native gettextinputdialogs, not open
> the qp. If the user wants to stay in tray mode, he/she should be allowed to do that (the
> export still uses the preview window, it's just the qp shouldn't be forced on the user or
> be misused)"

```gherkin
Given the QuickPanel is closed
When the user picks Add project (or Quick add) from the tray menu
Then a native text-input dialog appears
And submitting it adds/tracks the project
And the QuickPanel stays closed throughout
```

### Flow 2: About as an independent window

> DEVNOTES.md: "Can the about be its own independent window, again as to not rely on QP."

```gherkin
Given the QuickPanel is closed
When the user picks About from the tray menu
Then the About content opens in its own window
And the QuickPanel stays closed
```

### Flow 3: Tray cannot change QuickPanel mode

> DEVNOTES.md: "It should not be possible to change the mode of the qp in the tray, only
> open the qp."

```gherkin
Given the tray menu is open
When the user looks for QuickPanel mode controls
Then none exist — only an item that opens the QuickPanel
```

**Out of scope for this epic:**
- Timesheet export UI — the preview window remains the export surface (the source note says
  so explicitly).
- Timesheet range correctness (EPIC-003) and opacity (EPIC-004).

---

## 2. Function Call Signatures

*(deferred to revision 2)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Flow | Authority | Notes |
|---|---|---|
| 1 | Legacy application output — the log file format spec (`architecture/description/03-domain-boundaries.md`, Session/Log Domain) | A project added via native dialog must produce log entries identical to one added via the QuickPanel |
| 2, 3 | Existing Playwright harness (`tests/ui/helpers/tauri.ts` mocks) for window-side effects; native tray behavior verified manually on real hardware | Tray menu construction is unit-testable in Rust (`tray_menu.rs`); dialog interaction is not |

### What is deliberately not tested

Native OS dialog rendering and tray menu appearance — manual verification only.

---

## 4. Checklist

To be broken into slices at formulation review; stub items:

```md
[ ] 1. Rust unit test: tray menu contains no QuickPanel-mode items — done when it fails against current tray_menu.rs
[ ] 2. Remove mode items from tray menu construction in `src-tauri/src/infrastructure/tray_menu.rs` — done when test 1 passes
[ ] 3. Native input dialog for Add project from tray (file TBD at formulation) — done when Flow 1 passes manually and the log output matches a QP-added project
[ ] 4. Same for Quick add — done when Flow 1 variant passes
[ ] 5. Standalone About window — done when Flow 2 passes with QP closed
```

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [x] Amends Description sections: `description/02-two-surfaces-one-domain.md` (tray feature list changes)
- [ ] **Requires a Constitution change**

### North star deviation

North star: "always there, never in the way … minimal friction to log a moment."

**No — it advances it.** Forcing the QuickPanel open to complete a tray action is exactly
the interference the north star forbids. This epic removes friction; it trades nothing away.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Two related fixes already exist on the unmerged branch `claude/app-architecture-rewrite-j2daqd`: feedback-email → GitHub-Issues link (`51d4c4c`) and tray timesheet window not showing (`d842074`), both also from DEVNOTES.md (lines 5–6). Merge/cherry-pick that branch, or re-implement on `dev`? | Yes — implementing Flow 1–3 on `dev` without deciding risks conflicting with that branch's tray changes | Before the first slice |
| Q2 | Which native dialog mechanism (Tauri dialog plugin is already a dependency) | No | During slice 3 |

### New capability

None — reshapes existing tray actions; no feature the north star doesn't already allude to.
