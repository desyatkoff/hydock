# Configuration

Example `~/.config/hydock/config.toml`:

```TOML
[config]

# Hide dock when unfocused
auto_hide = true

# Enable random order of app icons
# You don't actually want this, idk why did I add this
chaos_mode = false

# List of application class names that should never appear in the dock
ignore_applications = [
    "wofi"
]

# List of application class names that should always appear in the dock
pinned_applications = [
    "discord",
    "firefox",
    "kitty",
    "krita",
    "obsidian",
    "steam",
    "telegram",
    "thunar"
]
```
