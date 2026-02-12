# Troubleshooting

## 1. Service Is Running But Remaps Do Not Work

Check:

```bash
systemctl --user status keyrs.service --no-pager --full
journalctl --user -u keyrs.service -n 200 --no-pager
```

Then validate config:

```bash
~/.local/bin/keyrs --check-config --config ~/.config/keyrs/config.toml
```

If valid but still not remapping:
- verify target app `wm_class` in verbose logs
- verify condition regex matches real class/name
- verify no later keymap overrides with conflicting mapping

## 2. Config Applies But Wrong Mapping Fires

Use verbose run:

```bash
~/.local/bin/keyrs --config ~/.config/keyrs/config.toml --verbose
```

Look for:
- loaded keymaps and conditions
- matched combo output
- whether expected condition is `matches=true`

## 3. Keyboard Type Detection Is Wrong

Set explicit override in `settings.toml`:

```toml
[keyboard]
override_type = "Mac"
```

Restart service:

```bash
~/.local/bin/keyrs-service restart
```

## 4. NumPad Behavior Unexpected

`forced_numpad` affects numpad interpretation logic in engine conditions.

Set in `settings.toml`:

```toml
[features]
forced_numpad = true
```

Restart service and verify logs show loaded setting.

## 5. uinput / Permission Problems

Install udev rules:

```bash
scripts/keyrs-service.sh install-udev
```

Then replug keyboard or reboot/log out as needed.

## 6. TUI Changes Not Reflected

In TUI:
- press `s` to save
- press `a` to save + restart

Or manually restart:

```bash
~/.local/bin/keyrs-service restart
```

## 7. Compose Problems

Check modular source and generated file:

```bash
ls -1 ~/.config/keyrs/config.d
~/.local/bin/keyrs --compose-config ~/.config/keyrs/config.d --compose-output /tmp/keyrs.compose.toml
~/.local/bin/keyrs --check-config --config /tmp/keyrs.compose.toml
```

## 8. Fast Recovery (Known Good Config)

Keep a backup:

```bash
cp ~/.config/keyrs/config.toml ~/.config/keyrs/config.toml.bak
```

Rollback:

```bash
cp ~/.config/keyrs/config.toml.bak ~/.config/keyrs/config.toml
~/.local/bin/keyrs-service restart
```
