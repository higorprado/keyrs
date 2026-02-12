# Install And Service Management

## Quick Install

From repo root:

```bash
cargo build --release --features pure-rust --bin keyrs
scripts/keyrs-service.sh install
scripts/keyrs-service.sh install-udev
# or non-interactive:
scripts/keyrs-service.sh install --yes
scripts/keyrs-service.sh install-udev --yes
```

This will:
- install binary to `~/.local/bin/keyrs`
- copy config fragments from `config.d.example/` to `~/.config/keyrs/config.d/` (if missing)
- compose `~/.config/keyrs/config.toml` from `~/.config/keyrs/config.d/`
- validate generated config before service activation
- copy `settings.toml` into `~/.config/keyrs/` (if missing)
- install and enable user systemd service `keyrs.service`
- optionally install a machine-level udev rules file for `uinput`/keyboard access

## Commands

```bash
scripts/keyrs-service.sh install [--bin <path>] [--force] [--yes]
scripts/keyrs-service.sh uninstall
scripts/keyrs-service.sh install-udev [--yes]
scripts/keyrs-service.sh uninstall-udev [--yes]
scripts/keyrs-service.sh start
scripts/keyrs-service.sh stop
scripts/keyrs-service.sh restart
scripts/keyrs-service.sh status
```

## Notes

- `install` and `uninstall` show a confirmation summary before applying changes.
- Use `--yes` to skip the confirmation prompt (for automation).
- `install-udev` and `uninstall-udev` require root privileges (via root shell or `sudo`).

- Installer is idempotent.
- Existing `~/.config/keyrs/config.d/` and `settings.toml` are preserved unless `--force` is provided.
- `config.toml` is always regenerated from `~/.config/keyrs/config.d/` during install.
- `uninstall` removes the service unit but keeps binary/config by default.
- udev rules are installed at `/etc/udev/rules.d/99-keyrs.rules`.
- Use `--dry-run` to inspect actions without applying changes.

## Troubleshooting

- If `systemctl --user` is unavailable, ensure systemd user session is enabled.
- If service does not start, run:

```bash
systemctl --user status keyrs.service --no-pager --full
journalctl --user -u keyrs.service -n 100 --no-pager
```
