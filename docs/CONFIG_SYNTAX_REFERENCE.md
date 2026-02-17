# Config Syntax Reference

This document describes `config.toml` and modular `config.d/*.toml` syntax.

## Top-Level Sections

Supported root sections:

- `[general]`
- `[modmap.default]`
- `[[modmap.conditionals]]`
- `[[multipurpose]]`
- `[[keymap]]`
- `[timeouts]`
- `[devices]`
- `[delays]`
- `[window]`

Unknown fields are rejected by parser (`deny_unknown_fields`).

## 1. General

```toml
[general]
suspend_key = "F11"
diagnostics_key = "F12"
emergency_eject_key = "Pause"
```

## 2. Modmap

Global modifier/key-level remap.

### Default modmap

```toml
[modmap.default]
LEFT_META = "LEFT_CTRL"
RIGHT_META = "RIGHT_CTRL"
```

### Conditional modmap

```toml
[[modmap.conditionals]]
name = "my_conditional_modmap"
condition = "wm_class =~ '(?i)terminal|kitty'"
mappings = { CAPSLOCK = "LEFT_CTRL" }
```

## 3. Multipurpose (tap/hold)

```toml
[[multipurpose]]
name = "caps_tap_esc_hold_ctrl"
trigger = "CAPSLOCK"
tap = "ESC"
hold = "LEFT_CTRL"
condition = "wm_class =~ '(?i)kitty'"
```

## 4. Keymap

```toml
[[keymap]]
name = "terminal_remaps"
condition = "wm_class =~ '(?i)terminal|kitty'"

[keymap.mappings]
"Super-c" = "Ctrl-Shift-c"
"Super-v" = "Ctrl-Shift-v"
```

### Output forms

Each mapping value can be:

1. Single key/combo string
```toml
"Super-c" = "Ctrl-c"
```

2. Sequence list
```toml
"Super-k" = ["Shift-End", "Backspace"]
```

3. Sequence with actions
```toml
"Alt-Delete" = ["Combo(Esc)", "Delay(25)", "Combo(d)"]
```

4. Text output
```toml
"Super-F8" = "Text(hello world)"
```

5. Unicode output
```toml
"Super-u" = "Unicode(00E9)"
# or
"Super-u" = "U+00E9"
```

### When to use `Combo(...)` vs plain output

Use plain output when you want a direct output key while preserving currently held physical modifiers. Use `Combo(...)` inside a non-`bind` sequence when you need the emitted key/combo to be isolated from held modifiers.

```toml
# Plain output: direct key output. Held physical modifiers remain held.
"Super-j" = "Enter"

# Sequence combo step: explicit combo in a macro.
"Super-j" = ["Combo(Enter)"]
```

Rules of thumb:
- Plain string (`"Enter"`, `"Ctrl-c"`) preserves held modifiers.
- Non-`bind` sequences temporarily release held modifiers, run steps, then restore them.
- Therefore `"Alt-Down" = "Enter"` can be seen by apps as Alt+Enter while Alt is physically held.
- `"Alt-Down" = ["Combo(Enter)"]` isolates Enter from held Alt in that step.
- `bind` is the opposite behavior: it keeps held modifiers for subsequent combo step(s).
- `["Enter"]` is still treated like plain single-key output, not as a `Combo(...)` step.

## 5. Sequence Actions

Supported in sequence arrays:

- `Combo(<combo>)`
- plain combo string (ex: `"Ctrl-c"`)
- `Delay(<ms>)`
- `Text(...)`
- `SetSetting(name=true|false)` (or `Set(name=on/off)`)
- `bind`
- `Ignore`

### `bind` semantics

`bind` changes how modifier state is handled for subsequent combo step(s), preserving held modifiers for correct app-native shortcuts in some flows.

## 6. Condition Language

Conditions are evaluated against runtime context.

Common fields:
- `wm_class`
- `wm_name`
- `device_name`/device predicates (depending on context)
- lock state predicates (e.g. `numlk`, `capslk`)
- settings flags (`settings.<name>`)

Common operators:
- regex match: `=~`
- boolean: `and`, `or`, `not`
- parentheses grouping

Examples:

```toml
condition = "wm_class =~ '(?i)kitty|alacritty' and settings.Enter2Ent_Cmd"
condition = "not (wm_class =~ '(?i)code')"
```

## 7. Timeouts

```toml
[timeouts]
multipurpose = 400
suspend = 1000
```

Parser ranges:
- `multipurpose`: 100..5000 ms
- `suspend`: 100..10000 ms

## 8. Device Filter

```toml
[devices]
only = ["AT Translated Set 2 keyboard", "Telink Wireless Gaming Keyboard"]
```

If omitted, keyboards are autodetected.

## 9. Output Delays

```toml
[delays]
key_pre_delay_ms = 0
key_post_delay_ms = 0
```

Allowed range (current parser): `0..150 ms` each.

## 10. Window Polling

`[window]` controls how often keyrs polls input events and refreshes active window context.

```toml
[window]
poll_timeout_ms = 100
update_interval_ms = 500
idle_sleep_ms = 10
```

Fields:

- `poll_timeout_ms`
Purpose: timeout passed to evdev poll loop.
Lower values reduce latency but can increase wakeups/CPU.
Range: `1..5000 ms`.
Default: `100`.

- `update_interval_ms`
Purpose: interval between `update_from_window_manager()` calls.
Lower values detect app/window switches faster.
Range: `10..10000 ms`.
Default: `500`.

- `idle_sleep_ms`
Purpose: sleep time in no-event fallback path.
Lower values can improve responsiveness during idle/error paths but may increase CPU.
Range: `0..1000 ms`.
Default: `10`.

Recommended baseline:

```toml
[window]
poll_timeout_ms = 60
update_interval_ms = 150
idle_sleep_ms = 5
```

## 11. Validation

Always validate before runtime:

```bash
~/.local/bin/keyrs --check-config --config ~/.config/keyrs/config.toml
```
