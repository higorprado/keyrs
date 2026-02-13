# Profile Guide

This guide explains how to use, create, and share keyrs configuration profiles.

## What is a Profile?

A profile is a pre-configured set of keyboard mappings designed for a specific use case or preference. Profiles allow you to:

- Quickly set up keyrs with sensible defaults
- Switch between different keyboard configurations
- Share your configurations with others
- Start from a known-good base configuration

## Using Profiles

### Listing Available Profiles

```bash
keyrs-service list-profiles
```

This shows all built-in profiles with their descriptions and tags.

### Viewing Profile Details

```bash
keyrs-service show-profile --profile <name>
```

Shows detailed information about a specific profile including its metadata and config files.

### Installing with a Profile

During initial installation:

```bash
keyrs-service install --profile <name>
```

This installs keyrs and applies the selected profile's configuration.

### Switching Profiles

To switch to a different profile on an existing installation:

```bash
keyrs-service profile-set <name>
```

This will:
1. Back up your current config
2. Copy the profile's config files
3. Validate the configuration
4. Restart the service

Use `--dry-run` to preview changes without applying them:

```bash
keyrs-service profile-set <name> --dry-run
```

### Installing from URL

You can install a profile from a URL (ZIP or TAR.GZ archive):

```bash
keyrs-service profile-set --url https://example.com/my-profile.tar.gz
```

## Built-in Profiles

| Profile | Description | Best For |
|---------|-------------|----------|
| `none` | Empty slate | Custom configurations from scratch |
| `minimal` | Clipboard + tab navigation | Simple setup, basic shortcuts |
| `ergonomic` | Caps→Esc/Ctrl, Enter→Ctrl | Reducing finger strain |
| `mac-standard` | macOS-style Super key shortcuts | macOS migrants |
| `mac-power` | Full app-specific configurations | Power users from macOS |
| `windows-standard` | Windows-style shortcuts | Windows migrants |
| `chromebook` | Chromebook-style mappings | Chromebook users |
| `developer` | IDE + terminal optimized | Software developers |
| `writer` | Writing-focused mappings | Authors, bloggers |
| `gamer` | Minimal interference | Gaming without remapping conflicts |
| `accessibility` | Simplified, longer timeouts | Accessibility needs |

## Profile Structure

A profile is a directory containing:

```
my-profile/
├── profile.toml      # Required: metadata
└── config.d/         # Required: configuration files
    ├── 000_base.toml
    ├── 100_applications.toml
    └── ...
```

### profile.toml Format

```toml
name = "my-profile"
description = "A brief description of what this profile does"
version = "1.0.0"
author = "Your Name"
tags = ["tag1", "tag2", "tag3"]
```

Fields:
- `name` (required): Profile identifier, should match directory name
- `description` (required): Brief description shown in listings
- `version` (required): Semantic version (e.g., "1.0.0")
- `author` (optional): Profile creator
- `tags` (optional): Array of tags for searching/filtering

### config.d/ Directory

Contains TOML configuration files that will be copied to `~/.config/keyrs/config.d/`. Files are processed in alphabetical order, so use numeric prefixes:

- `000_base.toml` - Base configuration, modmaps, timeouts
- `100_applications.toml` - Application-specific mappings
- `200_terminals.toml` - Terminal-specific overrides
- `500_editors.toml` - Editor/IDE overrides
- `900_fallback.toml` - Default fallback mappings

See [CONFIG_SYNTAX_REFERENCE.md](./CONFIG_SYNTAX_REFERENCE.md) for configuration syntax.

## Creating a Profile

### Step 1: Create Directory Structure

```bash
mkdir -p my-profile/config.d
```

### Step 2: Create Metadata

Create `profile.toml`:

```toml
name = "my-profile"
description = "My custom profile for specific workflow"
version = "1.0.0"
author = "Your Name"
tags = ["custom", "workflow"]
```

### Step 3: Add Configuration

Create config files in `config.d/`:

```bash
# Start with base configuration
cat > my-profile/config.d/000_base.toml << 'EOF'
[general]
suspend_key = "ControlRight"

[modmap.default]
CapsLock = "Esc"
EOF
```

### Step 4: Test Locally

```bash
# Validate the profile
keyrs --validate --config-dir ./my-profile/config.d

# Or set it
keyrs-service profile-set ./my-profile
```

### Step 5: Share

Package and share your profile:

```bash
# Create archive
tar -czvf my-profile.tar.gz my-profile/

# Share the archive file
```

## Sharing Profiles

### Creating a Shareable Archive

```bash
tar -czvf profile-name.tar.gz profile-name/
```

Or as ZIP:

```bash
zip -r profile-name.zip profile-name/
```

### Hosting Profiles

Profiles can be hosted anywhere accessible via HTTP/HTTPS:
- GitHub releases
- Personal website
- Cloud storage with public links

Users can install directly:

```bash
keyrs-service profile-set --url https://example.com/profile.tar.gz
```

### Profile Repository

Consider submitting your profile to the keyrs profile repository (if available) for community sharing.

## Customizing Profiles

After installing a profile, you can customize it:

### Option 1: Edit Installed Config

```bash
# Edit config files
nano ~/.config/keyrs/config.d/900_custom.toml

# Validate
keyrs --validate

# Apply changes
keyrs-service restart
```

### Option 2: Layer Additional Config

Create additional config files with higher numeric prefixes:

```bash
# Add personal customizations
cat > ~/.config/keyrs/config.d/950_personal.toml << 'EOF'
[[keymap]]
keys = ["Super", "Space"]
action = "LaunchApplication"
target = "rofi -show drun"
EOF
```

### Option 3: Create Derived Profile

Copy a built-in profile and modify:

```bash
# Copy built-in profile
cp -r profiles/developer profiles/my-developer

# Edit as needed
nano profiles/my-developer/profile.toml
nano profiles/my-developer/config.d/000_base.toml

# Install your derived profile
keyrs-service profile-set my-developer
```

## Profile Management

### Viewing Current Profile

Your current configuration is in `~/.config/keyrs/config.d/`. To see which profile you're using:

```bash
# Check for profile marker
cat ~/.config/keyrs/.profile 2>/dev/null || echo "No profile marker found"
```

### Backing Up Configuration

Before switching profiles:

```bash
# Automatic backup (done by profile-set)
ls ~/.config/keyrs/backups/

# Manual backup
cp -r ~/.config/keyrs/config.d ~/.config/keyrs/config.d.backup.$(date +%Y%m%d)
```

### Restoring Configuration

```bash
# From automatic backup
keyrs-service restore-backup

# From manual backup
cp -r ~/.config/keyrs/config.d.backup.YYYYMMDD/* ~/.config/keyrs/config.d/
keyrs-service restart
```

## Best Practices

### Profile Design

1. **Start minimal**: Include only essential mappings
2. **Document thoroughly**: Use clear descriptions in profile.toml
3. **Use sensible defaults**: Follow established keyboard conventions
4. **Test thoroughly**: Validate before sharing

### Configuration Organization

1. **Use numeric prefixes**: Ensures consistent load order
2. **Separate concerns**: Different files for different purposes
3. **Comment complex mappings**: Help users understand your choices
4. **Provide examples**: Include example keymaps for common customizations

### Version Control

1. **Version your profiles**: Update version in profile.toml
2. **Keep a changelog**: Track changes between versions
3. **Test upgrades**: Verify configs work with new keyrs versions

## Troubleshooting

### Profile Not Found

```bash
# Check available profiles
keyrs-service list-profiles

# Verify profile name spelling
keyrs-service show-profile --profile <name>
```

### Configuration Errors

```bash
# Validate configuration
keyrs --validate

# Check for syntax errors
keyrs --validate --verbose
```

### Profile Not Applying

```bash
# Check service status
systemctl --user status keyrs

# View logs
journalctl --user -u keyrs -f

# Manual restart
keyrs-service restart
```

## See Also

- [CONFIG_SYNTAX_REFERENCE.md](./CONFIG_SYNTAX_REFERENCE.md) - Full configuration syntax
- [INSTALL_AND_SERVICE.md](./INSTALL_AND_SERVICE.md) - Installation guide
- [../profiles/README.md](../profiles/README.md) - Built-in profiles documentation
