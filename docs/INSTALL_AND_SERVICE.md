# Install And Service

## Requirements

- Linux Wayland session
- `systemd --user`
- Rust toolchain (for building)
- `uinput` available in kernel

## Build

```bash
cd ~/Code/keyrs
cargo build --release --features pure-rust --bin keyrs --bin keyrs-tui
```

## Install

From `~/Code/keyrs`:

```bash
scripts/keyrs-service.sh install
```

What install does:
- installs `keyrs` to `~/.local/bin/keyrs`
- installs `keyrs-tui` to `~/.local/bin/keyrs-tui` (if built)
- installs runtime controller to `~/.local/bin/keyrs-service`
- copies default modular config from `config.d.example/` (if missing)
- composes `config.d` -> `config.toml`
- validates generated config
- installs and enables `keyrs.service`

## Udev Rules

For stable keyboard/uinput access:

```bash
scripts/keyrs-service.sh install-udev
```

This installs rules at:
- `/etc/udev/rules.d/99-keyrs.rules`

Remove with:

```bash
scripts/keyrs-service.sh uninstall-udev
```

## Service Commands

During development (repo script):

```bash
scripts/keyrs-service.sh status
scripts/keyrs-service.sh start
scripts/keyrs-service.sh stop
scripts/keyrs-service.sh restart
scripts/keyrs-service.sh apply-config
```

Runtime installed helper:

```bash
~/.local/bin/keyrs-service status
~/.local/bin/keyrs-service restart
~/.local/bin/keyrs-service apply-config
```

## Safe Update Flow

1. Edit `~/.config/keyrs/config.d/*.toml`
2. Run:

```bash
~/.local/bin/keyrs-service apply-config
```

This will:
- compose to temporary config
- run `--check-config`
- replace `~/.config/keyrs/config.toml` only if valid
- restart service

## Uninstall Service

From repo script:

```bash
scripts/keyrs-service.sh uninstall
```

This removes/disable the service unit. Config and binaries are typically kept unless script options specify otherwise.

## Useful Flags

- `--yes`: non-interactive confirmation
- `--dry-run`: preview actions without changing system
- `--force`: overwrite default copied files during install

## Troubleshooting

- If `systemctl --user` is unavailable, ensure systemd user session is enabled.
- If service does not start, run:

```bash
systemctl --user status keyrs.service --no-pager --full
journalctl --user -u keyrs.service -n 100 --no-pager
```

For more diagnostic guidance, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
