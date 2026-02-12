# settings.toml Reference

`settings.toml` controls feature flags and runtime behavior used by keyrs conditions and logic.

Default location:
- `~/.config/keyrs/settings.toml`

## File Structure

```toml
[features]
Enter2Ent_Cmd = true
Caps2Esc_Cmd = true
forced_numpad = true

[layout]
optspec_layout = "ABC"

[keyboard]
override_type = "Apple"
```

## Sections

## `[features]`

Dictionary of boolean flags.

- Missing flags default to `false`.
- Flags are available in conditions as `settings.<name>`.

Examples:

```toml
[features]
Enter2Ent_Cmd = true
DesktopGnome = false
```

Used in keymap conditions:

```toml
condition = "settings.Enter2Ent_Cmd and wm_class =~ '(?i)nemo|nautilus'"
```

### Common Feature Flags In Production Config

- `Enter2Ent_Cmd`
- `Caps2Esc_Cmd`
- `Caps2Cmd`
- `forced_numpad`
- `media_arrows_fix`
- `multi_lang`
- distro/desktop selectors:
  - `DistroFedoraGnome`
  - `DistroPop`
  - `DistroUbuntuOrFedoraGnome`
  - `DesktopBudgie`
  - `DesktopCosmicOrPop`
  - `DesktopGnome`
  - `DesktopKde`
  - `DesktopPantheon`
  - `DesktopSway`
  - `DesktopXfce`

## `[layout]`

Currently:
- `optspec_layout = "ABC" | "US"`

Used by special-character/output behavior paths.

## `[keyboard]`

- `override_type` (optional)

If set, it bypasses auto keyboard detection.

Typical values:
- `Apple` or `Mac`
- `Windows`
- `IBM`
- `Chromebook`

If unset, keyrs tries to auto-detect from connected keyboard devices.

## Boolean Value Parsing

The parser accepts booleans and common equivalents:

- true values: `true`, `"true"`, `"yes"`, `"on"`, `1`
- false values: `false`, `"false"`, `"no"`, `"off"`, `0`

## Runtime Behavior

- keyrs loads settings at startup.
- conditions can directly reference settings (`settings.X`).
- action steps can mutate settings at runtime (`SetSetting(...)`) inside sequences.

Example runtime mutation in config:

```toml
"Enter" = ["SetSetting(Enter2Ent_Cmd=false)", "F2"]
```

## Recommended Strategy

- Keep persistent environment toggles in `settings.toml`.
- Use `SetSetting(...)` in config only for workflow state machines (like file-manager mode toggles).

## Validation Tips

- watch loaded values in verbose logs:

```bash
~/.local/bin/keyrs --config ~/.config/keyrs/config.toml --verbose
```

- edit + restart:

```bash
~/.local/bin/keyrs-service restart
```
