# keyrs Documentation

keyrs is a Rust-based keyboard remapper for Wayland. It is designed for people who need **consistent shortcuts across Linux apps**, especially when coming from macOS/Command-key muscle memory.

## Acknowledgment

This project was heavily inspired by Toshy. Without Toshy, this project would not have been possible.

## Installation

If you just want to install and run:

```bash
cd ~/Code/keyrs
cargo build --release --features pure-rust --bin keyrs --bin keyrs-tui
scripts/keyrs-service.sh install
scripts/keyrs-service.sh install-udev
~/.local/bin/keyrs-service status
```

For config updates later:

```bash
~/.local/bin/keyrs-service apply-config
```

## What keyrs is useful for

- make `Super`/`Command` shortcuts feel natural on Linux
- use different behavior per app/window (terminal vs IDE vs browser)
- keep large remap setups maintainable with modular config
- avoid risky manual deploys with compose+validate+restart workflow
- toggle environment behavior quickly with `settings.toml` or TUI

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

## Docs index

- `docs/INSTALL_AND_SERVICE.md`
  Full install, service lifecycle, udev setup, update flow.

- `docs/SHORTCUT_CONFIGURATION_GUIDE.md`
  Practical guide to configure shortcuts, precedence, conflict avoidance, and testing patterns.

- `docs/CONFIG_SYNTAX_REFERENCE.md`
  Full syntax reference for sections, actions, conditions, and validation.

- `docs/CONDITION_PATTERNS.md`
  Copy-paste ready condition patterns for terminals, browsers, file managers, and more.

- `docs/CONFIG_EXAMPLES.md`
  Real-world configuration examples for common use cases.

- `docs/SETTINGS_REFERENCE.md`
  Complete `settings.toml` reference and runtime semantics.

- `docs/CONFIG_COMPOSE_WORKFLOW.md`
  How modular config composition works and how to apply safely.

- `docs/TROUBLESHOOTING.md`
  Logs, diagnostics, and common failure fixes.
