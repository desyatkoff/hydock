# Styling

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

#app-icon, #app-launcher {
    border-radius: 8px;
    padding: 4px 8px;
    transition: background-color 0.25s ease;
}

#app-icon:hover, #app-launcher:hover {
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
