# Shortcut Configuration Guide

This guide focuses on practical shortcut configuration in keyrs.

## Mental model

Think in this order:

1. **Modmap**: raw key-to-key remap (global or conditional)
2. **Multipurpose**: tap/hold behavior on one key
3. **Keymap**: combo mappings with optional app conditions
4. **Sequence actions**: delays, text, setting toggles, bind semantics

For most shortcut work, you will use `[[keymap]]`.

## Minimal keymap

```toml
[[keymap]]
name = "global_basic"

[keymap.mappings]
"Super-c" = "Ctrl-c"
"Super-v" = "Ctrl-v"
```

## App-specific shortcuts

Terminal often needs different copy/paste combos:

```toml
[[keymap]]
name = "terminal_shortcuts"
condition = "wm_class =~ '(?i)terminal|kitty|alacritty|wezterm'"

[keymap.mappings]
"Super-c" = "Ctrl-Shift-c"
"Super-v" = "Ctrl-Shift-v"
```

Editor/browser fallback:

```toml
[[keymap]]
name = "editor_browser_shortcuts"
condition = "wm_class =~ '(?i)code|cursor|firefox|chrome|chromium'"

[keymap.mappings]
"Super-c" = "Ctrl-c"
"Super-v" = "Ctrl-v"
"Super-f" = "Ctrl-f"
```

## Sequences for complex behaviors

Delete to end of line:

```toml
"Super-k" = ["Shift-End", "Backspace"]
```

Terminal word delete style:

```toml
"Alt-Delete" = ["Combo(Esc)", "Delay(25)", "Combo(d)"]
```

Browser macro example:

```toml
"Super-comma" = ["Ctrl-l", "Delay(120)", "Text(about:preferences)", "Delay(160)", "Enter"]
```

## Stateful behavior with settings toggles

Use `SetSetting(...)` for mode-like flows:

```toml
"Enter" = ["SetSetting(Enter2Ent_Cmd=false)", "F2"]
"Esc" = ["SetSetting(Enter2Ent_Cmd=true)", "Esc"]
```

And gate keymaps by that setting:

```toml
condition = "wm_class =~ '(?i)nemo|nautilus' and settings.Enter2Ent_Cmd"
```

## Precedence and ordering

In modular config (`config.d`), files are composed in sorted filename order.

Recommended naming:
- `000_base.toml`
- `100_terminals.toml`
- `200_editors.toml`
- `300_filemanagers.toml`
- `900_fallback.toml`

Keep specific rules earlier and generic fallback rules later, or vice versa based on your tested precedence strategy. The important part is: keep ordering intentional and documented.

## Avoiding conflicts

- Avoid duplicate mappings for the same combo in overlapping conditions unless you intentionally rely on precedence.
- Keep condition regexes explicit (`(?i)^kitty$` is safer than broad fuzzy matching when needed).
- Document why each app-specific exception exists.

## Recommended workflow for safe changes

1. Edit `~/.config/keyrs/config.d/*.toml`
2. Apply safely:

```bash
~/.local/bin/keyrs-service apply-config
```

3. If debugging matching behavior:

```bash
~/.local/bin/keyrs --config ~/.config/keyrs/config.toml --verbose
```

4. If needed, validate directly:

```bash
~/.local/bin/keyrs --check-config --config ~/.config/keyrs/config.toml
```

## Test checklist for each new shortcut

- Press in target app where it should map.
- Press in non-target app where it should not map.
- Press repeatedly (fast) to detect repeat/press-release issues.
- Switch focus between windows and retest.
- Confirm no conflict with fallback keymaps.

## Common patterns you will likely reuse

- Cmd-like copy/paste by context
- Terminal special mappings
- App-launcher macros with `Text(...)`
- state-machine toggles with `SetSetting(...)`
- desktop/distro-specific branches using settings flags

## When to use settings vs conditions

- Use **conditions** for window/app context (`wm_class`, `wm_name`).
- Use **settings flags** for environment or user-mode toggles (desktop distro, keyboard mode, workflow mode).

## Final advice

Prefer small focused keymaps over giant mixed blocks. It is easier to debug one intent per keymap than many unrelated mappings in one table.
