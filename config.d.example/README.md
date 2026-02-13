# Modular Configuration Directory

This directory contains example configuration fragments for keyrs. Each `.toml` file is loaded in alphabetical order and merged into a single configuration.

## Quick Start

1. Copy `minimal/` or specific files to `~/.config/keyrs/config.d/`
2. Run `keyrs-service apply-config` to compose and apply
3. Edit files as needed for your setup

## File Naming Convention

Files are processed in alphabetical order. Use numeric prefixes to control precedence:

| Prefix | Purpose |
|--------|---------|
| `000_` | Base modmaps (modifier remapping, global settings) |
| `100-199` | High-priority app-specific keymaps |
| `200-499` | Application-specific keymaps |
| `500-899` | Desktop environment overrides |
| `900_` | Fallback keymaps (lowest precedence) |

Lower-numbered files take precedence over higher-numbered ones when keymaps conflict.

## Directory Structure

```
~/.config/keyrs/config.d/
├── 000_base_modmap.toml      # Global modifier mappings
├── 100_terminal.toml         # Terminal-specific shortcuts
├── 200_browser.toml          # Browser-specific shortcuts
├── 300_editor.toml           # IDE/editor shortcuts
├── 900_fallback.toml         # Default keymaps (catch-all)
└── ...
```

## How Merging Works

When files are composed:

1. **Tables merge by key**: Later values override earlier ones
   - `[general]`, `[timeouts]`, `[devices]` sections merge

2. **Arrays append**: All entries from all files are combined
   - `[[keymap]]`, `[[multipurpose]]`, `[[modmap.conditionals]]` all stack

3. **Precedence**: Within the same array, earlier entries match first
   - First matching keymap wins
   - Use file naming to control priority

## Common Condition Patterns

### Window Class Matching

Match specific applications by their WM class:

```toml
# Terminals
condition = "wm_class =~ '(?i)alacritty|kitty|wezterm|gnome-terminal|konsole'"

# Browsers
condition = "wm_class =~ '(?i)firefox|chromium|brave|google-chrome'"

# File managers
condition = "wm_class =~ '(?i)nautilus|dolphin|thunar|nemo|pcmanfm'"

# VS Code and variants
condition = "wm_class =~ '(?i)code|code-oss|vscodium|cursor'"
```

### Excluding Applications

Use `not` to exclude:

```toml
# GUI apps (not terminals)
condition = "not (wm_class =~ '(?i)alacritty|kitty|terminal')"
```

### Settings-Based Conditions

Reference features from `settings.toml`:

```toml
condition = "settings.Enter2Ent_Cmd"
condition = "settings.Caps2Esc_Cmd and not (keyboard_type =~ 'Chromebook')"
```

### Keyboard Type

Different behavior for different keyboards:

```toml
condition = "keyboard_type =~ 'Mac'"
condition = "keyboard_type =~ 'Windows'"
condition = "keyboard_type =~ 'Chromebook'"
```

## Minimal Example

For a simple setup, use the `minimal/` directory:

```bash
cp -r config.d.example/minimal/* ~/.config/keyrs/config.d/
keyrs-service apply-config
```

## Full Example

For the complete production configuration with all app-specific mappings:

```bash
cp config.d.example/*.toml ~/.config/keyrs/config.d/
keyrs-service apply-config
```

## See Also

- `docs/CONFIG_SYNTAX_REFERENCE.md` - Full syntax documentation
- `docs/CONFIG_COMPOSE_WORKFLOW.md` - Detailed merge behavior
- `config.simple.toml` - Single-file alternative
