# Install And Service Management

## Quick Install

From repo root:

```bash
cargo build --release --features pure-rust --bin keyrs
scripts/keyrs-service.sh install
```

This will:
- install binary to `~/.local/bin/keyrs`
- copy default `config.toml` and `settings.toml` into `~/.config/keyrs/` (if missing)
- install and enable user systemd service `keyrs.service`

## Commands

```bash
scripts/keyrs-service.sh install [--bin <path>] [--force]
scripts/keyrs-service.sh uninstall
scripts/keyrs-service.sh start
scripts/keyrs-service.sh stop
scripts/keyrs-service.sh restart
scripts/keyrs-service.sh status
```

## Notes

- Installer is idempotent.
- Existing user config files are preserved unless `--force` is provided.
- `uninstall` removes the service unit but keeps binary/config by default.
- Use `--dry-run` to inspect actions without applying changes.

## Troubleshooting

- If `systemctl --user` is unavailable, ensure systemd user session is enabled.
- If service does not start, run:

```bash
systemctl --user status keyrs.service --no-pager --full
journalctl --user -u keyrs.service -n 100 --no-pager
```
