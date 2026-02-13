# keyrs Profiles

Pre-made configuration profiles for keyrs keyboard remapper.

## What is a Profile?

A profile is a complete keyboard configuration that includes:
- Modifier key remappings (modmap)
- Keyboard shortcuts (keymap)
- Multipurpose keys (tap/hold behavior)
- Application-specific rules

## Available Profiles

### Minimal (Starting Points)

| Profile | Description |
|---------|-------------|
| `none` | Empty slate - only suspend key, no remapping |
| `minimal` | Essential shortcuts - clipboard and tab management |
| `ergonomic` | Reduce strain - Caps→Esc/Ctrl, Enter→Ctrl |

### Migration (Platform Converts)

| Profile | Description |
|---------|-------------|
| `mac-standard` | macOS style - Super acts like Cmd with standard shortcuts |
| `mac-power` | Full macOS experience with app-specific optimizations |
| `windows-standard` | Windows migrants - familiar Ctrl key behavior |
| `chromebook` | Chrome OS style keyboard layout |

### Workflow (Usage Patterns)

| Profile | Description |
|---------|-------------|
| `developer` | IDE and terminal optimized, vim-friendly |
| `writer` | Content creation focus, text navigation |
| `gamer` | Minimal remapping to avoid game conflicts |
| `accessibility` | Simplified mappings, reduced complexity |

## Profile Details

### none

The empty slate. Only includes the suspend key (F11) to disable all remapping in emergencies. Perfect for users who want complete control and will build their own configuration from scratch.

### minimal

Lightest touch for essential consistency:
- Super+C/V/X for clipboard (copy/paste/cut)
- Super+A for select all
- Super+Z for undo
- Super+S for save
- Super+T/W for tab management

### ergonomic

Reduce finger strain and improve comfort:
- Caps Lock: tap = Escape, hold = Ctrl
- Enter: tap = Enter, hold = Ctrl
- Navigation shortcuts (Home/End, word delete)

### mac-standard

For macOS users switching to Linux:
- Super key behaves like Cmd on Mac
- Standard macOS shortcuts (Super+C/V/X/Z/A/S/Q)
- Works across GUI applications
- Proper terminal behavior preserved

### mac-power

The complete macOS experience:
- Everything in mac-standard
- Terminal-specific mappings (all major terminals)
- Browser-specific mappings (Chrome, Firefox, etc.)
- Editor-specific mappings (VSCode, JetBrains)
- File manager mappings
- Desktop environment integration

### windows-standard

For Windows migrants:
- Ctrl key behavior matching Windows
- Alt+Tab preserved for window switching
- Minimal interference with existing habits

### chromebook

Chrome OS keyboard layout:
- Search key as Super
- Ctrl+Alt arrows for desktop navigation
- Chromebook keyboard type detection

### developer

Optimized for software development:
- Mac-style base mappings
- Caps→Esc/Ctrl for vim users
- Terminal-specific optimizations
- IDE/editor mappings (VSCode, JetBrains)
- All major terminals supported

### writer

Content creation focus:
- Clipboard essentials
- Text navigation (Home/End, document navigation)
- Bold/Italic/Underline shortcuts
- Minimal interference with writing flow

### gamer

Minimal interference for gaming:
- Only suspend key functionality
- Native keyboard behavior preserved
- No modifier remapping that could affect games

### accessibility

Simplified keyboard experience:
- Longer multipurpose timeout (300ms)
- Caps→Esc/Ctrl and Enter→Ctrl multipurpose
- Core shortcuts only
- Reduced complexity for easier learning

## Using Profiles

### During Installation

```bash
# Interactive selection
keyrs-service install

# Direct profile selection
keyrs-service install --profile mac-standard

# From custom URL
keyrs-service install --profile-url https://example.com/my-profile.tar.gz
```

### After Installation

```bash
# List available profiles
keyrs-service list-profiles

# Show profile details
keyrs-service show-profile mac-power

# Switch to a different profile
keyrs-service switch-profile developer

# Switch from URL
keyrs-service switch-profile --url https://example.com/custom-profile.tar.gz
```

## Creating Custom Profiles

### Profile Structure

```
my-profile/
├── profile.toml       # Required: metadata
└── config.d/          # Required: configuration files
    ├── 000_base.toml
    ├── 100_multipurpose.toml
    └── 900_fallback.toml
```

### profile.toml Format

```toml
name = "my-profile"
description = "A custom profile for my workflow"
version = "1.0.0"
author = "your-name"
tags = ["custom", "personal"]
```

### Configuration Files

Files in `config.d/` are loaded in alphabetical order:
- `000_` - Base settings, modmaps
- `100_` - Multipurpose keys
- `200_` - Terminal-specific
- `300_` - File manager-specific
- `400_` - Browser-specific
- `500_` - Editor-specific
- `600_` - Desktop environment-specific
- `900_` - Fallback defaults

### Sharing Profiles

1. Create a directory with your profile
2. Include `profile.toml` and `config.d/`
3. Package as `.tar.gz` archive
4. Host on GitHub, GitLab, or any web server
5. Share the URL

Users can install with:
```bash
keyrs-service install --profile-url https://your-domain.com/my-profile.tar.gz
```

## Profile Archive Format

For URL-based profiles, create a `.tar.gz` archive:

```bash
tar -czvf my-profile.tar.gz my-profile/
```

The archive must contain:
- `profile.toml` at the root or one level deep
- `config.d/` directory with `.toml` files

## Validation

Before applying, profiles are validated:
1. `profile.toml` exists and is valid TOML
2. `config.d/` exists and contains `.toml` files
3. Composed config passes `keyrs --validate`

Invalid profiles are rejected with clear error messages.
