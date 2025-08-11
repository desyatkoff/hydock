# Hydock

```
 _   _           _            _    
| | | |_   _  __| | ___   ___| | __
| |_| | | | |/ _` |/ _ \ / __| |/ /
|  _  | |_| | (_| | (_) | (__|   < 
|_| |_|\__, |\__,_|\___/ \___|_|\_\
       |___/                       
```

<https://github.com/user-attachments/assets/de0d51aa-4f44-44c6-8d69-a972482ac78c>

## Description

Hydock is a Rust + GTK dock that uses Hyprland IPC

## Table of Contents

1. [Hydock](#hydock)
2. [Description](#description)
3. [Table of Contents](#table-of-contents)
4. [Features](#features)
5. [Installation](#installation)
6. [Customizing](#customizing)
7. [Feedback](#feedback)
8. [Contributing](#contributing)
9. [License](#license)

## Features

* Configuration support (`~/.config/hydock/config.toml`)
    + App launcher command
    + Auto-hide
    + Chaos mode
    + Ignore applications
    + Pinned applications
    + Show app launcher
    + Show separator
* Style support (`~/.config/hydock/style.css`)
    + Global window style
    + Dock style
    + Application icon style
    + Application launcher style
    + Application dots box style
    + Application dot style
    + Separator style
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

## Customizing

Refer to [CONFIGURATION.md](/docs/CONFIGURATION.md) and [STYLING.md](/docs/STYLING.md)

## Feedback

Found a bug? [Open an issue](https://github.com/desyatkoff/hydock/issues/new)

Want to request a feature? [Start a discussion](https://github.com/desyatkoff/hydock/discussions/new?category=ideas)

## Contributing

Refer to [CONTRIBUTING.md](/docs/CONTRIBUTING.md)

## License

Copyright (C) Sergey Desyatkov

Hydock is licensed under the GNU General Public License v3.0 or later. See the [LICENSE](LICENSE) file for more details
