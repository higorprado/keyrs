# keyrs Documentation

keyrs is a Rust-based keyboard remapper for Wayland. It is designed for people who need **consistent shortcuts across Linux apps**, especially when coming from macOS/Command-key muscle memory.

## Acknowledgment

This project was heavily inspired by Toshy. Without Toshy, this project would not have been possible.

## Quick Install

```bash
# Clone and build
git clone https://github.com/yourusername/keyrs.git
cd keyrs
cargo build --release

# Install binaries and service
scripts/keyrs-service.sh install
scripts/keyrs-service.sh install-udev

# Start using a profile (optional)
keyrs-service profile-set mac-standard
```

After initial setup, use the installed helper:

```bash
keyrs-service status          # Check service status
keyrs-service apply-config    # Apply config changes
keyrs-service profile-set <name>  # Switch profiles
```

## What keyrs is useful for

- make `Super`/`Command` shortcuts feel natural on Linux
- use different behavior per app/window (terminal vs IDE vs browser)
- keep large remap setups maintainable with modular config
- avoid risky manual deploys with compose+validate+restart workflow
- toggle environment behavior quickly with `settings.toml` or TUI

## Profiles

keyrs includes 11 pre-made profiles for common use cases. Switch between them quickly:

```bash
# List available profiles
keyrs-service list-profiles

# Switch to a profile
keyrs-service profile-set mac-standard

# Interactive selection
keyrs-service profile-select
```

Available profiles: `none`, `minimal`, `ergonomic`, `mac-standard`, `mac-power`, `windows-standard`, `chromebook`, `developer`, `writer`, `gamer`, `accessibility`

See [docs/PROFILE_GUIDE.md](docs/PROFILE_GUIDE.md) for creating and sharing custom profiles.

## Why keyrs was created

Linux shortcut behavior is inconsistent across desktop environments and apps. A single static remap is usually not enough.

Typical real-world mismatch:
- Terminal copy needs `Ctrl-Shift-c`
- VSCode copy needs `Ctrl-c`
- Browser/file-manager shortcuts have their own edge cases

keyrs solves this with context-aware keymaps, conditions, stateful actions, and safe deployment tooling.

## Why this architecture

- Rust runtime: stable binary behavior, strong parser/engine correctness, no Python runtime dependency for production path.
- TOML config: readable, structured, and easy to split into modules.
- `settings.toml` separate from `config.toml`: environment toggles are isolated from mapping rules.
- `config.d` compose flow: manageable modular files, deterministic order, safer maintenance.
- service helper: prevents broken config rollout by validating before replacing runtime config.

## Runtime paths

- Binary: `~/.local/bin/keyrs`
- Service helper: `~/.local/bin/keyrs-service`
- Optional TUI: `~/.local/bin/keyrs-tui`
- User config folder: `~/.config/keyrs/`
- Runtime config file: `~/.config/keyrs/config.toml`
- Modular config folder: `~/.config/keyrs/config.d/`
- Runtime settings file: `~/.config/keyrs/settings.toml`
- User service unit: `~/.config/systemd/user/keyrs.service`

## Documentation

- [Profile Guide](docs/PROFILE_GUIDE.md) — List, switch, create, and share profiles
- [Install & Service](docs/INSTALL_AND_SERVICE.md) — Full install, service lifecycle, udev setup
- [Shortcut Configuration](docs/SHORTCUT_CONFIGURATION_GUIDE.md) — Configure shortcuts, avoid conflicts
- [Config Syntax Reference](docs/CONFIG_SYNTAX_REFERENCE.md) — Full TOML syntax reference
- [Condition Patterns](docs/CONDITION_PATTERNS.md) — Copy-paste patterns for apps
- [Config Examples](docs/CONFIG_EXAMPLES.md) — Real-world configuration examples
- [Settings Reference](docs/SETTINGS_REFERENCE.md) — `settings.toml` reference
- [Config Compose Workflow](docs/CONFIG_COMPOSE_WORKFLOW.md) — Modular config system
- [Troubleshooting](docs/TROUBLESHOOTING.md) — Logs, diagnostics, common fixes
