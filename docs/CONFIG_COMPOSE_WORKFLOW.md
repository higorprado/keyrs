# Config Compose Workflow (config.d)

keyrs supports modular config through composition.

## Why modular config

A giant `config.toml` becomes hard to maintain. `config.d` allows splitting by topic/app.

Typical split:
- `000_general.toml`
- `100_filemanagers.toml`
- `200_terminals.toml`
- `300_browsers.toml`
- `900_fallback.toml`

## Compose Command

```bash
~/.local/bin/keyrs --compose-config ~/.config/keyrs/config.d --compose-output ~/.config/keyrs/config.toml
```

If `--compose-output` is omitted, default output is parent of source dir + `config.toml`.

## Merge Rules

During compose:
- TOML files are read in sorted filename order.
- `[general]` and `[timeouts]`: table entries merged.
- `[modmap.default]`: entries merged.
- `[[modmap.conditionals]]`: appended.
- `[[multipurpose]]`: appended.
- `[[keymap]]`: appended.
- unknown top-level sections are inserted/overwritten by last file.

## Recommended Naming Convention

Use prefixes for order:
- `000_...`
- `100_...`
- `200_...`
- `900_...`

This makes precedence obvious and deterministic.

## Safe Production Flow

Prefer service helper command:

```bash
~/.local/bin/keyrs-service apply-config
```

It does:
1. compose to temp file
2. validate with `--check-config`
3. replace runtime config only if valid
4. restart service

## Manual Flow

```bash
~/.local/bin/keyrs --compose-config ~/.config/keyrs/config.d --compose-output /tmp/keyrs.test.toml
~/.local/bin/keyrs --check-config --config /tmp/keyrs.test.toml
cp /tmp/keyrs.test.toml ~/.config/keyrs/config.toml
~/.local/bin/keyrs-service restart
```
