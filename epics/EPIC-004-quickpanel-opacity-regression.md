# EPIC-004: QuickPanel opacity regression

**Status:** draft
**Created:** 2026-07-12
**Architecture baseline:** 299b448

Source: migrated from `DEVNOTES.md` (deleted in the same commit).

---

## 1. BDD — User Flows

### Flow 1: Opacity applies to the whole window

> DEVNOTES.md: "The opacity does not work anymore, the ui gets toned down, but the
> background remains."

```gherkin
Given the user sets QuickPanel opacity below 100%
When the QuickPanel is visible
Then the entire window — content and background — renders at that opacity
And the setting survives an app restart
```

**Out of scope for this epic:**
- Any new opacity range or per-element opacity controls.

---

## 2. Function Call Signatures

*(deferred to revision 2)*

---

## 3. TDD — Testing Strategy

### Authority for correctness

| Flow | Authority | Fixture | Tolerance |
|---|---|---|---|
| 1 | Legacy application output — opacity previously worked (the note says "anymore"); the regression window can be found in git history where the behavior changed | Playwright: assert the CSS/window-level opacity mechanism is applied to the root element, via `tests/ui/helpers/tauri.ts` mocks | Exact — the opacity value set equals the value applied |

### What is deliberately not tested

Actual OS compositor rendering — the native window-transparency path is verified manually on
real hardware; the UI test pins the frontend's half of the contract.

---

## 4. Checklist

```md
[ ] 1. Bisect/inspect: identify where the background stopped receiving opacity (likely split between `src/views/screens/quickpanel.css` and the native window layer) — done when the broken layer is named with evidence
[ ] 2. Failing UI test pinning opacity application to the root element — done when it fails for the right reason
[ ] 3. Fix in the single file item 1 identified — done when test 2 passes and manual check on hardware confirms the background dims
```

---

## 5. Summary

### Architecture impact

- [x] No change to ARCHITECTURE.md expected
- [ ] Amends Description sections
- [ ] **Requires a Constitution change**

### North star deviation

North star: "always there, never in the way."

**No.** Opacity exists precisely so the panel can stay present without visually intruding;
fixing it restores that property.

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Is the regression frontend CSS or native window transparency? | No — checklist item 1 answers it | Slice 1 |

### New capability

None — regression fix.
