# EPIC-012: Timed QuickPanel pop-up reminder

**Status:** draft
**Created:** 2026-10-23
**Architecture baseline:** 84f7bd7

Source: team request (W&E, COWI) — people forget to switch project during the day, so the
hours they register later are guesses. An optional timer that brings the QuickPanel up lets
them confirm or switch the active project.

---

## 1. BDD — User Flows

### Flow 1: Choose a reminder interval from the tray

```gherkin
Given the tray menu is open
When the user picks "Pop-up reminder" → "Every 30 minutes", "Every hour" or "Every 2 hours"
Then that entry is checked in the submenu
And the choice survives an app restart
```

### Flow 2: The QuickPanel pops up on the interval

```gherkin
Given a reminder interval is set
When that interval has passed since the interval was chosen, the app started, or the last pop-up
Then the QuickPanel window is shown and focused
And the next pop-up comes one interval later
```

### Flow 3: Off by default

```gherkin
Given a user who never chose an interval (or picked "Off")
When any amount of time passes
Then the QuickPanel never pops up on its own
```

**Out of scope for this epic:**
- A QuickPanel control for the interval — the tray is enough to prove the feature; add one if
  users ask.
- Pop-ups aligned to the clock (on the hour), custom intervals, working-hours or weekday limits.
- Skipping the pop-up while a project is active — the point is to confirm the active project.

---

## 2. Function Call Signatures

```rust
// models/domain/settings.rs — persisted, serde default 0 so existing settings.json loads unchanged
pub reminder_interval_minutes: u32, // 0 = off

// controllers/settings_controller.rs
pub fn set_reminder_interval(minutes: u32, state: &AppState, app: &AppHandle)

// app_setup.rs
fn reminder_due(minutes: u32, elapsed: Duration) -> bool
```

---

## 3. TDD — Testing Strategy

### Test map

| Flow | Function call | Authority | Fixture | Tolerance |
|---|---|---|---|---|
| 1 | `settings::save` / `settings::load` | Critical regression path 7: settings survive restart | `settings.rs` `save_and_load_roundtrip` | Exact |
| 2 | `reminder_due` | This epic's Flow 2 | `app_setup.rs` tests | Exact, in whole seconds |
| 3 | `settings::load` of a pre-EPIC-012 `settings.json` | This epic's Flow 3 | `settings.rs` `load_normalizes_empty_sort_mode` | Exact (`0`) |

### What is deliberately not tested

The timer thread and tray submenu wiring: they need a running Tauri app. Verified by hand by
temporarily setting `reminder_interval_minutes` to `1` in `settings.json`.

---

## 4. Checklist

```md
[x] 1. Add `reminder_interval_minutes` (default 0) in `src-tauri/src/models/domain/settings.rs` — done when `save_and_load_roundtrip` and `load_normalizes_empty_sort_mode` pass with it asserted
[x] 2. Add `set_reminder_interval` in `src-tauri/src/controllers/settings_controller.rs` — saves the setting and rebuilds the tray menu
[x] 3. Add the "Pop-up reminder" submenu in `src-tauri/src/infrastructure/tray_menu.rs` and its `reminder::` handler in `src-tauri/src/infrastructure/tray.rs`
[x] 4. Add `reminder_due` and the timer thread in `src-tauri/src/app_setup.rs` — done when the `reminder_due` tests pass
```

---

## 5. Summary

### Architecture impact

- [ ] No change to ARCHITECTURE.md expected
- [x] Amends Description sections: `description/02` (tray feature list)
- [ ] **Requires a Constitution change**

### North star deviation

North star: "always there, never in the way — a clock punch card that follows you through the
day without ever interrupting it."

**Yes, for users who opt in.** A pop-up interrupts by design. It is off by default, so nobody's
day changes unless they choose an interval; those who do trade a brief interruption for a record
they can trust, which serves the north star's other half ("unshakeable trust in what gets
reconstructed").

### Open questions

| # | Question | Blocks | Decision needed by |
|---|---|---|---|
| Q1 | Should the QuickPanel also expose the interval setting? | No | When users ask for it |

### New capability

None beyond the north star: it nudges people to log, it does not add a new kind of record.
