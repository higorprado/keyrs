# Condition Patterns Reference

This document provides copy-paste ready condition patterns for common use cases in keyrs configuration files.

## Condition Syntax Basics

Conditions are evaluated against runtime context using a simple expression language:

```toml
condition = "wm_class =~ '(?i)firefox|chrome'"
condition = "wm_class =~ '(?i)kitty' and settings.DesktopGnome"
condition = "not (wm_class =~ '(?i)terminal')"
```

### Common Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `=~` | Regex match | `wm_class =~ 'firefox'` |
| `and` | Logical AND | `condition1 and condition2` |
| `or` | Logical OR | `condition1 or condition2` |
| `not` | Logical NOT | `not (wm_class =~ 'terminal')` |

### Regex Tips

- `(?i)` - Case-insensitive matching (put at start of pattern)
- `^...$` - Anchor to start/end of string
- `|` - Alternation (OR) within regex
- `.*` - Match any characters

---

## Window Class Patterns

### Terminal Emulators

**All common terminals:**
```toml
condition = "wm_class =~ '(?i)terminal|alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty|wave|terminology|deepin-terminal|contour|hyper|kgx|cosmicterm|blackbox'"
```

**Specific terminals:**

| Terminal | Pattern |
|----------|---------|
| Alacritty | `wm_class =~ '(?i)^alacritty$'` |
| Kitty | `wm_class =~ '(?i)^kitty$'` |
| WezTerm | `wm_class =~ '(?i)wezterm'` |
| GNOME Terminal | `wm_class =~ '(?i)gnome-terminal'` |
| Konsole | `wm_class =~ '(?i)^konsole$|^org\.kde\.konsole$'` |
| XFCE Terminal | `wm_class =~ '(?i)^xfce4-terminal$'` |
| Ghostty | `wm_class =~ '(?i)ghostty'` |
| COSMIC Terminal | `wm_class =~ '(?i)^com\.system76\.cosmicterm$'` |
| Deepin Terminal | `wm_class =~ '(?i)^deepin-terminal$'` |
| Black Box | `wm_class =~ '(?i)com\.raggesilver\.blackbox'` |
| Console (GNOME) | `wm_class =~ '(?i)org\.gnome\.console|console'` |
| Elementary Terminal | `wm_class =~ '(?i)^io\.elementary\.terminal$'` |
| Hyper | `wm_class =~ '(?i)^hyper$'` |
| Contour | `wm_class =~ '(?i)^contour$'` |
| Terminology | `wm_class =~ '(?i)^terminology$'` |
| Wave | `wm_class =~ '(?i)^wave$'` |

**Not in terminal (GUI apps):**
```toml
condition = "not (wm_class =~ '(?i)terminal|alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty|wave|terminology|deepin-terminal|contour|hyper|kgx|cosmicterm|blackbox')"
```

---

### Web Browsers

**All common browsers:**
```toml
condition = "wm_class =~ '(?i)firefox|librewolf|waterfox|zen|chrom|chrome|vivaldi|falkon'"
```

**Specific browsers:**

| Browser | Pattern |
|---------|---------|
| Firefox | `wm_class =~ '(?i)firefox'` |
| LibreWolf | `wm_class =~ '(?i)librewolf'` |
| Waterfox | `wm_class =~ '(?i)waterfox'` |
| Zen Browser | `wm_class =~ '(?i)^zen$'` |
| Chrome/Chromium | `wm_class =~ '(?i)chrom|chrome'` |
| Google Chrome | `wm_class =~ '(?i)google-chrome'` |
| Chromium | `wm_class =~ '(?i)chromium'` |
| Brave | `wm_class =~ '(?i)brave'` |
| Microsoft Edge | `wm_class =~ '(?i)microsoft-edge'` |
| Vivaldi | `wm_class =~ '(?i)^vivaldi.*$'` |
| Falkon | `wm_class =~ '(?i)^org\.kde\.falkon$|^falkon$'` |

**Firefox family (Firefox, LibreWolf, Waterfox, Zen):**
```toml
condition = "wm_class =~ '(?i)firefox|librewolf|waterfox|zen'"
```

**Chromium family (Chrome, Chromium, Brave, Edge, Vivaldi):**
```toml
condition = "wm_class =~ '(?i)chrom|chrome|vivaldi|brave|edge'"
```

---

### File Managers

**All common file managers:**
```toml
condition = "wm_class =~ '(?i)nautilus|dolphin|thunar|nemo|pcmanfm|krusader|spacefm|caja|cosmic|peony'"
```

**Specific file managers:**

| File Manager | Pattern |
|--------------|---------|
| Nautilus (GNOME) | `wm_class =~ '(?i)^nautilus$|^org\.gnome\.Nautilus$'` |
| Dolphin (KDE) | `wm_class =~ '(?i)^dolphin$|^org\.kde\.dolphin$'` |
| Thunar (XFCE) | `wm_class =~ '(?i)^thunar$'` |
| Nemo (Cinnamon) | `wm_class =~ '(?i)^nemo$'` |
| PCManFM | `wm_class =~ '(?i)pcmanfm'` |
| PCManFM-Qt | `wm_class =~ '(?i)^pcmanfm-qt$'` |
| Krusader | `wm_class =~ '(?i)^krusader$'` |
| SpaceFM | `wm_class =~ '(?i)^spacefm$'` |
| Caja (MATE) | `wm_class =~ '(?i)^caja$'` |
| COSMIC Files | `wm_class =~ '(?i)^com\.system76\.cosmic\.Files$'` |
| Peony (Deepin) | `wm_class =~ '(?i)^peony-qt$|^peony$'` |
| Pantheon Files | `wm_class =~ '(?i)^io\.elementary\.files$'` |
| DDE File Manager | `wm_class =~ '(?i)^dde-file-manager$'` |

---

### Text Editors and IDEs

**All common editors:**
```toml
condition = "wm_class =~ '(?i)code|code-oss|vscodium|cursor|jetbrains|kate|kwrite|sublime|gedit|xed'"
```

**Specific editors:**

| Editor | Pattern |
|--------|---------|
| VSCode | `wm_class =~ '(?i)code'` |
| VSCode OSS | `wm_class =~ '(?i)code-oss'` |
| VSCodium | `wm_class =~ '(?i)vscodium'` |
| Cursor | `wm_class =~ '(?i)cursor'` |
| JetBrains IDEs | `wm_class =~ '(?i)jetbrains'` |
| JetBrains (excluding Toolbox) | `wm_class =~ '(?i)jetbrains' and not (wm_class =~ '(?i)toolbox')` |
| Kate | `wm_class =~ '(?i)^org\.kde\.kate$'` |
| KWrite | `wm_class =~ '(?i)^kwrite$|^org\.kde\.kwrite$'` |
| Sublime Text | `wm_class =~ '(?i)sublime'` |
| gedit | `wm_class =~ '(?i)^gedit$'` |
| xed (Linux Mint) | `wm_class =~ '(?i)^xed$'` |
| GNOME Text Editor | `wm_class =~ '(?i)^gnome-text-editor$|^org\.gnome\.texteditor$'` |
| LibreOffice Writer | `wm_class =~ 'libreoffice-writer'` |

**VSCode family:**
```toml
condition = "wm_class =~ 'code|code-oss|vscodium|cursor'"
```

**Not VSCode (for different word-wise navigation):**
```toml
condition = "not (wm_class =~ 'code|code-oss|vscodium|cursor')"
```

---

## Window Name Patterns

Window name (`wm_name`) matches the window title. Use for dialog-specific mappings.

### Common Dialog Patterns

| Dialog | Pattern |
|--------|---------|
| Settings/Preferences | `wm_name =~ '(?i)settings|preferences|options'` |
| Find/Replace | `wm_name =~ '(?i)find|replace|search'` |
| Save dialog | `wm_name =~ '(?i)save|save as'` |
| Close confirmation | `wm_name =~ '(?i)close.*document|unsaved.*changes'` |
| Vivaldi Settings | `wm_name =~ '(?i)^Vivaldi Settings:.*Vivaldi$'` |
| KWrite Close Dialog | `wm_name =~ '(?i)^Close Document.*KWrite$'` |

### Combined Class and Name

```toml
# Vivaldi settings dialog only
condition = "wm_class =~ '(?i)^vivaldi.*$' and wm_name =~ '(?i)^Vivaldi Settings:.*Vivaldi$'"

# KWrite close confirmation dialog
condition = "wm_class =~ '(?i)^kwrite$|^org\.kde\.kwrite$' and wm_name =~ '(?i)^Close Document.*KWrite$'"
```

---

## Settings-Based Conditions

Settings are defined in `settings.toml` and referenced as `settings.<Name>`.

### Desktop Environment Settings

| Setting | Description |
|---------|-------------|
| `settings.DesktopGnome` | GNOME desktop |
| `settings.DesktopGnomePre45` | GNOME before version 45 |
| `settings.DesktopKde` | KDE Plasma desktop |
| `settings.DesktopXfce` | XFCE desktop |
| `settings.DesktopCosmicOrPop` | COSMIC or Pop!_OS |
| `settings.DesktopBudgie` | Budgie desktop |
| `settings.DesktopPantheon` | Pantheon (Elementary OS) |
| `settings.DesktopSway` | Sway tiling WM |
| `settings.DesktopDeepin` | Deepin desktop |

### Distribution Settings

| Setting | Description |
|---------|-------------|
| `settings.DistroFedoraGnome` | Fedora with GNOME |
| `settings.DistroUbuntuOrFedoraGnome` | Ubuntu or Fedora with GNOME |
| `settings.DistroPop` | Pop!_OS |

### Feature Flags

| Setting | Description |
|---------|-------------|
| `settings.Enter2Ent_Cmd` | Enter key behaves as Command |
| `settings.Caps2Esc_Cmd` | Caps Lock as Escape/Command |
| `settings.Caps2Cmd` | Caps Lock as Command |
| `settings.forced_numpad` | Force numpad mode |
| `settings.media_arrows_fix` | Fix media arrow keys |
| `settings.multi_lang` | Multi-language keyboard |

### Combined Settings and Class

```toml
# Terminal on GNOME desktop
condition = "wm_class =~ '(?i)terminal|alacritty|kitty' and settings.DesktopGnome"

# GUI apps on Pop!_OS (not terminals)
condition = "settings.DistroPop and not (wm_class =~ '(?i)terminal')"

# Specific app with feature flag
condition = "settings.Enter2Ent_Cmd and wm_class =~ '(?i)nemo|nautilus'"
```

---

## Keyboard Type Conditions

Match the detected or configured keyboard type.

```toml
# Apple/Mac keyboard
condition = "keyboard_type == 'Apple'"

# Chromebook keyboard
condition = "keyboard_type == 'Chromebook'"

# Windows keyboard
condition = "keyboard_type == 'Windows'"
```

---

## Lock State Conditions

Match keyboard lock states.

```toml
# Num Lock is on
condition = "numlk"

# Caps Lock is on
condition = "capslk"

# Num Lock off
condition = "not numlk"
```

---

## Complex Condition Examples

### Terminal-Specific with Desktop

```toml
# All terminals on GNOME
condition = "wm_class =~ '(?i)terminal|alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty|wave|terminology|deepin-terminal|contour|hyper|kgx|cosmicterm|blackbox' and settings.DesktopGnome"
```

### GUI Apps Only on Specific Desktop

```toml
# GUI apps only on Pop!_OS
condition = "settings.DistroPop and not (wm_class =~ '(?i)terminal|alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty|wave|terminology|deepin-terminal|contour|hyper|kgx|cosmicterm|blackbox')"
```

### App-Specific Dialog

```toml
# Firefox settings page (detected by URL)
condition = "wm_class =~ '(?i)firefox' and wm_name =~ '(?i)preferences'"
```

### Excluding Specific Apps

```toml
# All browsers except Firefox
condition = "wm_class =~ '(?i)chrom|chrome|vivaldi|falkon' and not (wm_class =~ '(?i)firefox')"

# JetBrains IDEs but not Toolbox app
condition = "wm_class =~ '(?i)jetbrains' and not (wm_class =~ '(?i)toolbox')"
```

---

## Quick Reference Table

| Category | Core Pattern |
|----------|--------------|
| All Terminals | `wm_class =~ '(?i)terminal|alacritty|kitty|wezterm|gnome-terminal|konsole|xfce4-terminal|ghostty|wave|terminology|deepin-terminal|contour|hyper|kgx|cosmicterm|blackbox'` |
| All Browsers | `wm_class =~ '(?i)firefox|librewolf|waterfox|zen|chrom|chrome|vivaldi|falkon'` |
| All File Managers | `wm_class =~ '(?i)nautilus|dolphin|thunar|nemo|pcmanfm|krusader|spacefm|caja|cosmic|peony'` |
| All Editors | `wm_class =~ '(?i)code|code-oss|vscodium|cursor|jetbrains|kate|kwrite|sublime|gedit|xed'` |
| Not Terminal | `not (wm_class =~ '(?i)terminal|...')` |
| GNOME GUI | `settings.DesktopGnome and not (wm_class =~ '(?i)terminal|...')` |
