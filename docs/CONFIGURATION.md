# Configuration

Example `~/.config/hydock/config.toml`:

```TOML
[config]

# Shell command to execute when the app launcher is clicked
app_launcher_command = "rofi -show drun"

# Which icon to use for app launcher
app_launcher_icon = "applications-all-symbolic"

# Hide dock when unfocused
auto_hide = true

# Enable random order of app icons
# You don't actually want this, idk why did I add this
chaos_mode = false

# List of application class names that should never appear in the dock
ignore_applications = [
    "rofi"
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

# Add app launcher button on the right
show_app_launcher = true

# Add separator between apps and app launcher
show_separator = true
```
