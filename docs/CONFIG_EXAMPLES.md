# Configuration Examples

This document provides real-world configuration examples for common use cases. Each example is self-contained and can be adapted to your needs.

## Table of Contents

- [macOS Migration Setup](#macos-migration-setup)
- [Minimal Configuration](#minimal-configuration)
- [Application-Specific Examples](#application-specific-examples)
- [Desktop Environment Examples](#desktop-environment-examples)
- [Keyboard-Specific Examples](#keyboard-specific-examples)
- [Advanced Macro Examples](#advanced-macro-examples)
- [Troubleshooting Examples](#troubleshooting-examples)

---

## macOS Migration Setup

Complete setup for users transitioning from macOS who want familiar keyboard shortcuts.

### settings.toml

```toml
[features]
# Use Enter as Right Ctrl when held (for Cmd simulation)
Enter2Ent_Cmd = true
# Caps Lock as Escape (tap) / Ctrl (hold)
Caps2Esc_Cmd = true

[layout]
optspec_layout = "US"

[keyboard]
# Override detection if needed
# override_type = "Apple"
```

### config.d/000_modmap.toml

```toml
# Caps Lock behaves normally (multipurpose handles tap/hold)
[modmap.default]
CAPSLOCK = "CAPSLOCK"

# Enter as Right Ctrl when held
[[multipurpose]]
name = "enter_to_cmd"
trigger = "ENTER"
tap = "ENTER"
hold = "RIGHT_CTRL"
condition = "settings.Enter2Ent_Cmd"

# Caps Lock: tap=Escape, hold=Right Ctrl
[[multipurpose]]
name = "caps_to_esc"
trigger = "CAPSLOCK"
tap = "ESCAPE"
hold = "RIGHT_CTRL"
condition = "settings.Caps2Esc_Cmd"
```

### config.d/100_terminal.toml

```toml
# Terminal uses Ctrl+Shift for clipboard
[[keymap]]
name = "terminal_macos"
condition = "wm_class =~ '(?i)alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty'"

[keymap.mappings]
# Copy/Paste
"Super-c" = "Ctrl-Shift-c"
"Super-v" = "Ctrl-Shift-v"
# Tab management
"Super-t" = "Ctrl-Shift-t"
"Super-w" = "Ctrl-Shift-w"
# New window
"Super-n" = "Ctrl-Shift-n"
# Find
"Super-f" = "Ctrl-Shift-f"
# Line editing
"Super-Backspace" = "Ctrl-u"
"Super-Delete" = "Ctrl-k"
"Alt-Backspace" = "Ctrl-w"
```

### config.d/900_fallback.toml

```toml
# Default Super → Ctrl mapping for all GUI apps
[[keymap]]
name = "macos_fallback"
condition = "not (wm_class =~ '(?i)alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty')"

[keymap.mappings]
# Standard shortcuts
"Super-a" = "Ctrl-a"
"Super-c" = "Ctrl-c"
"Super-f" = "Ctrl-f"
"Super-n" = "Ctrl-n"
"Super-o" = "Ctrl-o"
"Super-p" = "Ctrl-p"
"Super-q" = "Ctrl-q"
"Super-r" = "Ctrl-r"
"Super-s" = "Ctrl-s"
"Super-t" = "Ctrl-t"
"Super-v" = "Ctrl-v"
"Super-w" = "Ctrl-w"
"Super-x" = "Ctrl-x"
"Super-y" = "Ctrl-y"
"Super-z" = "Ctrl-z"
# Emacs-style navigation
"Super-b" = "Left"
"Super-f" = "Right"
"Super-p" = "Up"
"Super-n" = "Down"
"Super-a" = "Home"
"Super-e" = "End"
"Super-k" = ["Shift-End", "Backspace"]
"Super-d" = "Delete"
```

---

## Minimal Configuration

The simplest working configuration for basic Super → Ctrl mapping.

```toml
# config.toml - Minimal setup

[general]
suspend_key = "F11"

[timeouts]
multipurpose = 200
suspend = 1000

# Single fallback keymap
[[keymap]]
name = "basic_super_to_ctrl"
condition = "not (wm_class =~ '(?i)terminal|alacritty|kitty|gnome-terminal|konsole')"

[keymap.mappings]
"Super-c" = "Ctrl-c"
"Super-v" = "Ctrl-v"
"Super-x" = "Ctrl-x"
"Super-z" = "Ctrl-z"
"Super-a" = "Ctrl-a"
"Super-s" = "Ctrl-s"
"Super-f" = "Ctrl-f"
"Super-q" = "Ctrl-q"
```

---

## Application-Specific Examples

### Firefox Configuration

```toml
# Firefox-specific mappings
[[keymap]]
name = "firefox"
condition = "wm_class =~ '(?i)firefox|librewolf|waterfox'"

[keymap.mappings]
# Open settings via URL bar
"Super-comma" = ["Ctrl-l", "Delay(120)", "Text(about:preferences)", "Delay(160)", "Enter"]
# Delete to line start/end in URL bar
"Super-Backspace" = ["Shift-Home", "Backspace"]
"Super-Delete" = ["Shift-End", "Delete"]
# Private window
"Shift-Super-n" = "Shift-Ctrl-p"
# Tab switching by number
"Super-1" = "Alt-1"
"Super-2" = "Alt-2"
"Super-3" = "Alt-3"
"Super-4" = "Alt-4"
"Super-5" = "Alt-5"
"Super-6" = "Alt-6"
"Super-7" = "Alt-7"
"Super-8" = "Alt-8"
"Super-9" = "Alt-9"
```

### VSCode Configuration

```toml
# VSCode and variants
[[keymap]]
name = "vscode"
condition = "wm_class =~ '(?i)code|code-oss|vscodium|cursor'"

[keymap.mappings]
# Standard operations
"Super-f" = "Ctrl-f"
"Super-c" = "Ctrl-c"
"Super-x" = "Ctrl-x"
# Quick fix
"Super-dot" = "Ctrl-dot"
# Terminal
"Super-Grave" = "Ctrl-Grave"
# Word navigation
"Alt-Left" = "Ctrl-Left"
"Alt-Right" = "Ctrl-Right"
"Alt-Shift-Left" = "Ctrl-Shift-Left"
"Alt-Shift-Right" = "Ctrl-Shift-Right"
# Line delete
"Super-Backspace" = ["Shift-Home", "Delete"]
"Super-Delete" = ["Shift-End", "Delete"]
```

### JetBrains IDE Configuration

```toml
# JetBrains (IntelliJ, PyCharm, WebStorm, etc.)
[[keymap]]
name = "jetbrains"
condition = "wm_class =~ '(?i)jetbrains' and not (wm_class =~ '(?i)toolbox')"

[keymap.mappings]
# Tool windows (0-9)
"Ctrl-Key_0" = "Alt-Key_0"
"Ctrl-Key_1" = "Alt-Key_1"
"Ctrl-Key_2" = "Alt-Key_2"
"Ctrl-Key_3" = "Alt-Key_3"
"Ctrl-Key_4" = "Alt-Key_4"
"Ctrl-Key_5" = "Alt-Key_5"
"Ctrl-Key_6" = "Alt-Key_6"
"Ctrl-Key_7" = "Alt-Key_7"
"Ctrl-Key_8" = "Alt-Key_8"
"Ctrl-Key_9" = "Alt-Key_9"
# Settings
"Ctrl-Comma" = "Ctrl-Alt-s"
# Navigation
"Super-Up" = "Alt-Up"
"Super-Down" = "Alt-Down"
"Ctrl-g" = "F3"
# Refactoring
"Ctrl-t" = "Ctrl-Alt-Shift-t"
# Debugging
"Super-r" = "Shift-F10"
"Super-d" = "Shift-F9"
```

### Dolphin File Manager (KDE)

```toml
# Dolphin file manager
[[keymap]]
name = "dolphin"
condition = "wm_class =~ '(?i)^dolphin$|^org\\.kde\\.dolphin$'"

[keymap.mappings]
# Navigation
"Super-Left" = "Alt-Left"
"Super-Right" = "Alt-Right"
"Super-Up" = "Alt-Up"
# Location bar
"Super-l" = "Ctrl-l"
# Find
"Super-f" = "Ctrl-f"
# New tab
"Super-t" = "Ctrl-t"
# Close tab
"Super-w" = "Ctrl-w"
# Delete to trash (with dialog)
"Super-Backspace" = "Delete"
```

---

## Desktop Environment Examples

### GNOME Desktop

```toml
# settings.toml
[features]
DesktopGnome = true

# config.d/600_gnome.toml
[[keymap]]
name = "gnome_gui"
condition = "settings.DesktopGnome and not (wm_class =~ '(?i)terminal')"

[keymap.mappings]
# Input source switch
"Super-Space" = "Shift-Ctrl-Space"
# Toggle fullscreen
"Ctrl-Super-f" = "Alt-F10"
# Screenshots
"Shift-Super-3" = "Shift-PRINT"
"Shift-Super-4" = "Alt-PRINT"
"Shift-Super-5" = "PRINT"
```

### KDE Plasma

```toml
# settings.toml
[features]
DesktopKde = true

# config.d/600_kde.toml
[[keymap]]
name = "kde_gui"
condition = "settings.DesktopKde and not (wm_class =~ '(?i)terminal')"

[keymap.mappings]
# Window operations
"Super-h" = "Alt-F3"
# Screenshot
"Shift-Super-3" = "Shift-PRINT"
"Shift-Super-4" = "Meta-Shift-S"
```

### Pop!_OS / COSMIC

```toml
# settings.toml
[features]
DistroPop = true
DesktopCosmicOrPop = true

# config.d/600_pop.toml
[[keymap]]
name = "pop_gui"
condition = "settings.DistroPop and not (wm_class =~ '(?i)terminal')"

[keymap.mappings]
# Input source
"Super-Space" = "Super-slash"
# Workspace navigation
"Super-Right" = ["bind", "Combo(Super-Ctrl-Up)"]
"Super-Left" = ["bind", "Combo(Super-Ctrl-Down)"]
```

---

## Keyboard-Specific Examples

### Chromebook Keyboard

```toml
# Chromebook-specific mappings
[[keymap]]
name = "chromebook"
condition = "keyboard_type == 'Chromebook'"

[keymap.mappings]
# Chromebook has no Super key, use Search
"Search-c" = "Ctrl-c"
"Search-v" = "Ctrl-v"
"Search-x" = "Ctrl-x"
"Search-a" = "Ctrl-a"
# Navigation
"Search-Left" = "Home"
"Search-Right" = "End"
"Search-Up" = "Ctrl-Home"
"Search-Down" = "Ctrl-End"
```

### Compact Keyboard (Numpad as Numbers)

```toml
# settings.toml
[features]
forced_numpad = true

# config.d/000_numpad.toml
[[modmap.conditionals]]
name = "forced_numpad"
condition = "settings.forced_numpad"
mappings = { KP1 = "KEY_1", KP2 = "KEY_2", KP3 = "KEY_3", KP4 = "KEY_4", KP5 = "KEY_5", KP6 = "KEY_6", KP7 = "KEY_7", KP8 = "KEY_8", KP9 = "KEY_9", KP0 = "KEY_0", KPDOT = "DOT", KPENTER = "ENTER" }
```

---

## Advanced Macro Examples

### Open Settings Macro

```toml
# Open browser settings via URL bar
[[keymap]]
name = "browser_settings_macro"
condition = "wm_class =~ '(?i)firefox|librewolf|waterfox'"

[keymap.mappings]
"Super-comma" = ["Ctrl-l", "Delay(120)", "Text(about:preferences)", "Delay(160)", "Enter"]
```

### Menu Navigation Macro

```toml
# Access menu via keyboard
[[keymap]]
name = "menu_macro"
condition = "wm_class =~ '(?i)^xfce4-terminal$'"

[keymap.mappings]
"Super-comma" = ["Alt-e", "Delay(100)", "e"]
```

### Toggle State Macro

```toml
# Toggle a setting and perform action
[[keymap]]
name = "toggle_macro"
condition = "settings.Enter2Ent_Cmd"

[keymap.mappings]
"Enter" = ["SetSetting(Enter2Ent_Cmd=false)", "F2"]
```

### Complex Sequence Macro

```toml
# Vivaldi keyboard shortcut help
[[keymap]]
name = "vivaldi_help_macro"
condition = "wm_class =~ '(?i)^vivaldi.*$'"

[keymap.mappings]
"Super-slash" = ["F10", "Delay(200)", "H", "Delay(200)", "K"]
```

---

## Troubleshooting Examples

### Ignore Specific Shortcuts

```toml
# Disable a shortcut that conflicts
[[keymap]]
name = "ignore_conflict"
condition = "wm_class =~ '(?i)gnome-text-editor'"

[keymap.mappings]
# This app doesn't support this shortcut
"Super-slash" = ["Ignore"]
```

### Pass-Through with Bind

```toml
# Preserve modifier state for app-native handling
[[keymap]]
name = "passthrough"
condition = "wm_class =~ '(?i)^hyper$'"

[keymap.mappings]
# Let the app handle Ctrl+Tab natively
"Ctrl-Tab" = ["bind", "Combo(Ctrl-Tab)"]
"Shift-Ctrl-Tab" = ["bind", "Combo(Shift-Ctrl-Tab)"]
```

### App-Specific Dialog Handling

```toml
# Handle dialog within same app
[[keymap]]
name = "dialog_close"
condition = "wm_class =~ '(?i)^vivaldi.*$' and wm_name =~ '(?i)^Vivaldi Settings:.*Vivaldi$'"

[keymap.mappings]
# Close settings dialog with Escape
"Esc" = "Alt-F4"
```

### Unicode Output

```toml
# Type special characters
[[keymap]]
name = "unicode_chars"

[keymap.mappings]
"Super-u-e" = "Unicode(00E9)"  # é
"Super-u-a" = "U+00E0"         # à
"Super-u-o" = "Unicode(00F6)"  # ö
```

---

## Validation Commands

Always validate your configuration:

```bash
# Check syntax
keyrs --check-config --config ~/.config/keyrs/config.toml

# Compose modular config
keyrs --compose-config ~/.config/keyrs/config.d --compose-output /tmp/test.toml

# Run with verbose logging
keyrs --config ~/.config/keyrs/config.toml --verbose
```

## Common Issues

### Shortcut Not Working

1. Check if the app uses a different shortcut
2. Verify `wm_class` matches (use `xprop WM_CLASS` or `wmctrl -lx`)
3. Check for conflicting keymaps (lower-numbered files take precedence)
4. Ensure condition syntax is correct

### Wrong Window Class

```bash
# Find the correct window class
xprop WM_CLASS
# Then click on the target window

# Or list all windows
wmctrl -lx
```

### Modifier Not Detected

1. Check `keyboard_type` detection in verbose logs
2. Set `override_type` in `settings.toml` if needed
3. Verify the key codes in `modmap` are correct

---

## See Also

- [CONDITION_PATTERNS.md](CONDITION_PATTERNS.md) - Copy-paste ready condition patterns
- [SHORTCUT_CONFIGURATION_GUIDE.md](SHORTCUT_CONFIGURATION_GUIDE.md) - Practical shortcut configuration guide
- [CONFIG_SYNTAX_REFERENCE.md](CONFIG_SYNTAX_REFERENCE.md) - Full syntax reference
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Debugging and diagnostics
