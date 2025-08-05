# Hydock

```
 _   _           _            _    
| | | |_   _  __| | ___   ___| | __
| |_| | | | |/ _` |/ _ \ / __| |/ /
|  _  | |_| | (_| | (_) | (__|   < 
|_| |_|\__, |\__,_|\___/ \___|_|\_\
       |___/                       
```

## Description

<p align="center"><img src="assets/preview.png"/></p>

Hydock is a Rust + GTK dock that uses Hyprland IPC

## Table of Contents

1. [Hydock](#hydock)
2. [Description](#description)
3. [Table of Contents](#table-of-contents)
4. [Features](#features)
5. [Installation](#installation)
6. [Configuration](#configuration)
7. [Styling](#styling)
8. [Usage](#usage)
9. [Feedback](#feedback)
10. [Contributing](#contributing)
11. [License](#license)

## Features

* Configuration support (`~/.config/hydock/config.toml`)
    + Auto-hide
    + Chaos mode
    + Ignore applications
    + Pinned applications
* Style support (`~/.config/hydock/style.css`)
    + Global window style
    + Dock style
    + Application icon style
    + Application dots box style
    + Application dot style
* Refreshes every second
    + Configuration settings reload
    + Style properties reload
    + Open applications update
    + Application window dots update (based on windows count)

## Installation

Choose your preferred installation method:

* `git clone` the repository and launch installer script
    ```Shell
    git clone https://github.com/desyatkoff/hydock.git && cd hydock/ && bash ./install.sh
    ```
* `curl` the installer script
    ```Shell
    bash <(curl -fsSL https://raw.githubusercontent.com/desyatkoff/hydock/main/install.sh)
    ```

## Configuration

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

## Styling

Example `~/.config/hydock/style.css`:

```CSS
hydock {
    border-radius: 8px;
    color: #cdd6f4;
    font-family: "Noto Sans";
    font-size: 16px;
    margin-bottom: 8px;
    min-height: 58px;
}

#dock {
    background-color: #1e1e2e;
    border-radius: 8px;
    border: 2px solid #11111b;
    padding: 4px;
}

#app-icon {
    border-radius: 8px;
    padding: 4px 8px;
    transition: background-color 0.25s ease;
}

#app-icon:hover {
    background-color: #313244;
}

#app-dots-box {
    margin-top: 2px;
}

#app-dot {
    background-color: #89b4fa;
    border-radius: 50%;
}
```

## Usage

Currently, functionality is very minimal, so...

* Launch
    ```Shell
    hydock
    ```

## Feedback

Found a bug? [Open an issue](https://github.com/desyatkoff/hydock/issues/new)

Want to request a feature? [Start a discussion](https://github.com/desyatkoff/hydock/discussions/new?category=ideas)

## Contributing

Refer to [CONTRIBUTING.md](/docs/CONTRIBUTING.md)

## License

Copyright (C) Sergey Desyatkov

Hydock is licensed under the GNU General Public License v3.0 or later. See the [LICENSE](LICENSE) file for more details
