/*
 * This file is part of Hydock
 *
 * Copyright (C) 2025 Sergey Desyatkov
 *
 * Hydock is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License,
 * or (at your option) any later version
 *
 * Hydock is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details
 *
 * You should have received a copy of the GNU General Public License
 * along with Hydock. If not, see <https://www.gnu.org/licenses/>
 */

use glib::{
    timeout_add_seconds_local,
    ControlFlow
};
use gtk4::{
    prelude::*,
    Align,
    Application,
    ApplicationWindow,
    Box as GtkBox,
    CssProvider,
    EventControllerMotion,
    GestureClick,
    Image,
    Orientation,
    Separator,
    STYLE_PROVIDER_PRIORITY_USER,
    style_context_add_provider_for_display
};
use gtk4::gdk::Display;
use gtk4_layer_shell::{
    Edge,
    Layer,
    LayerShell
};
use serde::{
    Deserialize,
    Serialize
};
use std::{
    collections::HashMap,
    env,
    fs,
    io,
    process::{
        Command,
        Output
    },
    rc::Rc,
    thread
};

/// Small two-variant enum to avoid excessive `bool`s in a struct, while staying TOML-compatible (serializes/deserializes as `bool`)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Toggle {
    Off,
    On
}

impl Default for Toggle {
    fn default() -> Self {
        Toggle::Off
    }
}

impl<'de> Deserialize<'de> for Toggle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let b = bool::deserialize(deserializer)?;

        Ok(if b { Toggle::On } else { Toggle::Off })
    }
}

impl Serialize for Toggle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        serializer.serialize_bool(matches!(self, Toggle::On))
    }
}

/// Wrapper for the full Hydock configuration
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    config: ConfigSettings
}

/// Config settings loaded from `config.toml`
///
/// * `app_launcher_command`: Shell command to execute when the app launcher is clicked
/// * `app_launcher_icon`: Which icon to use for app launcher
/// * `auto_hide`: Hide dock when unfocused
/// * `chaos_mode`: Enable random order of app icons
/// * `dock_position`: Anchor of the dock panel
/// * `ignore_applications`: List of application class names that should never appear in the dock
/// * `override_app_icons`: Which icons to use for specified class names
/// * `pinned_applications`: List of application class names that should always appear in the dock
/// * `show_app_launcher`: Add app launcher button on the right
/// * `show_separator`: Add separator between apps and app launcher
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigSettings {
    app_launcher_command: String,
    app_launcher_icon: String,
    auto_hide: Toggle,
    chaos_mode: Toggle,
    dock_position: String,
    ignore_applications: Vec<String>,
    override_app_icons: HashMap<String, String>,
    pinned_applications: Vec<String>,
    show_app_launcher: Toggle,
    show_separator: Toggle
}

/// Implements default config settings
impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            app_launcher_command: "rofi -show drun".into(),
            app_launcher_icon: "applications-all-symbolic".into(),
            auto_hide: Toggle::Off,
            chaos_mode: Toggle::Off,
            dock_position: "bottom".into(),
            ignore_applications: Vec::new(),
            override_app_icons: HashMap::new(),
            pinned_applications: Vec::new(),
            show_app_launcher: Toggle::On,
            show_separator: Toggle::On
        }
    }
}

/// Represents a Hyprland client (a window) with its application class name
#[derive(Debug, Deserialize)]
struct HyprlandClient {
    class: String
}

/// Entry point
fn main() {
    let app = Application::builder()
        .application_id("com.github.desyatkoff.hydock")
        .build();
    app.connect_activate(build_dock);
    app.run();
}

/// Loads dock
fn build_dock(app: &Application) {
    // Base Hydock GTK window
    let hydock = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .title("Hydock")
        .css_name("hydock")
        .can_focus(false)
        .build();
    hydock.init_layer_shell();

    let cfg_for_init = load_config();
    let (edge, dock_orientation, dots_orientation) = parse_dock_position(&cfg_for_init);

    hydock.set_anchor(edge, true);
    hydock.set_layer(Layer::Top);
    hydock.set_namespace(Some("hydock"));
    hydock.auto_exclusive_zone_enable();

    // Trigger for showing dock again after it became hidden (when `auto_hide` is `true`)
    let trigger = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .title("Hydock Trigger")
        .can_focus(false)
        .build();
    trigger.init_layer_shell();
    trigger.set_anchor(edge, true);
    trigger.set_layer(Layer::Top);

    if dock_orientation == Orientation::Horizontal {
        trigger.set_default_width(2147483647);
        trigger.set_default_height(1);
    } else {
        trigger.set_default_width(1);
        trigger.set_default_height(2147483647);
    }

    trigger.show();

    // Dock panel itself
    let dock = Rc::new(GtkBox::new(dock_orientation, 0));
    let dock_clone = Rc::clone(&dock);
    dock.set_widget_name("dock");
    hydock.set_child(Some(&*dock));

    // Main loop for refreshing dock
    timeout_add_seconds_local(1, move || {
        // Load config once per tick
        let cfg = load_config();
        let (edge, dock_orientation, dots_orientation) = parse_dock_position(&cfg);

        // Refresh layer anchors in case position changed
        hydock.set_anchor(edge, true);
        trigger.set_anchor(edge, true);

        let hydock_for_leave = hydock.clone();
        let hydock_motion = EventControllerMotion::new();
        hydock_motion.connect_leave(move |_| {
            hydock_for_leave.hide();
        });

        let hydock_for_show = hydock.clone();
        let trigger_motion = EventControllerMotion::new();
        trigger_motion.connect_enter(move |_, _, _| {
            hydock_for_show.show();
        });

        if cfg.auto_hide == Toggle::On {
            trigger.clone().show();

            hydock.add_controller(hydock_motion);
            trigger.add_controller(trigger_motion);
        } else {
            hydock.clone().show();
            trigger.clone().hide();

            hydock.remove_controller(&hydock_motion);
            trigger.remove_controller(&trigger_motion);
        }

        load_style();

        // Clear and rebuild dock children
        while let Some(child) = dock_clone.first_child() {
            dock_clone.remove(&child);
        }

        build_apps(&dock_clone, &cfg, dock_orientation, dots_orientation);

        if cfg.show_app_launcher == Toggle::On {
            build_app_launcher(&dock_clone, &cfg, dots_orientation);
        }

        ControlFlow::Continue
    });
}

/// Loads app icons & dots
fn build_apps(dock: &Rc<GtkBox>, cfg: &ConfigSettings, dock_orientation: Orientation, dots_orientation: Orientation) {
    let mut counts = collect_app_counts(cfg);

    // Collect apps into a Vector
    let mut entries: Vec<(String, usize)> = counts.drain().collect();

    // Sort app icons in alphabetical order if `chaos_mode` is `Off`
    if cfg.chaos_mode == Toggle::Off {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    // Add app icons & dots
    for (class, count) in entries {
        let app_widget = build_app_widget(&class, count, cfg, dock_orientation, dots_orientation);
        dock.append(&app_widget);
    }
}

/// Gets all application window count into a HashMap containing class name and window count
fn collect_app_counts(cfg: &ConfigSettings) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    // Add actually opened apps
    for client in fetch_hyprland_clients() {
        *counts.entry(client.class.to_lowercase()).or_insert(0) += 1;
    }

    // Ensure pinned apps appear in dock even if they have no open windows
    for pinned in &cfg.pinned_applications {
        counts.entry(pinned.to_lowercase()).or_insert(0);
    }

    // Remove unwanted apps
    for ignored in &cfg.ignore_applications {
        counts.remove(&ignored.to_lowercase());
    }

    counts
}

/// Renders application icon widget for dock
fn build_app_widget(
    class: &str,
    count: usize,
    cfg: &ConfigSettings,
    dock_orientation: Orientation,
    dots_orientation: Orientation,
) -> GtkBox {
    // Icons lookup
    let app_icon = Image::from_icon_name(class);
    app_icon.set_pixel_size(32);

    if let Some(override_icon) = cfg.override_app_icons.get(class) {
        app_icon.set_icon_name(Some(override_icon));
    }

    if app_icon.icon_name().is_none() {
        app_icon.set_icon_name(Some("application-default-icon"));
    }

    let apps_wrapper = GtkBox::new(dots_orientation, 0);
    apps_wrapper.set_widget_name("app-icon");
    apps_wrapper.append(&app_icon);

    // Click & middle-click
    let class_owned = class.to_owned();
    let apps_gesture = GestureClick::builder().button(0).build();
    apps_gesture.connect_pressed(move |gesture, _, _, _| {
        match gesture.current_button() {
            1 => {
                // Focus or launch
                if let Err(e) = focus_or_launch(&class_owned) {
                    eprintln!("`focus_or_launch` error for {class_owned}: {e}");
                }
            },
            2 => {
                // Close or launch
                if let Err(e) = close_or_launch(&class_owned) {
                    eprintln!("`close_or_launch` error for {class_owned}: {e}");
                }
            },
            _ => {}
        }
    });
    apps_wrapper.add_controller(apps_gesture);

    // Represent app's window count using dots
    let app_dots_box = GtkBox::new(dock_orientation, 4);
    app_dots_box.set_widget_name("app-dots-box");
    app_dots_box.set_halign(Align::Center);
    app_dots_box.set_valign(Align::Center);

    for _ in 0..count {
        let app_dot = GtkBox::new(dots_orientation, 0);
        app_dot.set_widget_name("app-dot");
        app_dot.set_size_request(4, 4);
        app_dots_box.append(&app_dot);
    }

    apps_wrapper.append(&app_dots_box);

    apps_wrapper
}

/// Focuses the first window of specified app class
///
/// Launches this app if failed to focus
fn focus_or_launch(class: &str) -> io::Result<()> {
    let address = first_client_address_for_class(class)?;

    if !address.is_empty() {
        let cmd = format!("hyprctl dispatch focuswindow address:{address}");
        let out = run_sh(&cmd)?;

        if String::from_utf8_lossy(&out.stdout).trim() == "No such window found" {
            launch_background(&format!("/usr/bin/{class}"))?;
        }
    } else {
        launch_background(&format!("/usr/bin/{class}"))?;
    }

    Ok(())
}

/// Closes the first window of specified app class
///
/// Launches this app if failed to close
fn close_or_launch(class: &str) -> io::Result<()> {
    let address = first_client_address_for_class(class)?;

    if !address.is_empty() {
        let cmd = format!("hyprctl dispatch closewindow address:{address}");
        let out = run_sh(&cmd)?;

        if String::from_utf8_lossy(&out.stdout).trim() == "No such window found" {
            launch_background(&format!("/usr/bin/{class}"))?;
        }
    } else {
        launch_background(&format!("/usr/bin/{class}"))?;
    }

    Ok(())
}

/// Launches a command and waits in a detached background thread to avoid zombies
fn launch_background(cmd_path: &str) -> io::Result<()> {
    let mut child = Command::new(cmd_path).spawn()?;

    thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(())
}

/// Gets address of the first client with specified class
fn first_client_address_for_class(class: &str) -> io::Result<String> {
    let cmd = format!("hyprctl clients -j | jq -r '[.[] | select(.class == \"{class}\")][0].address'");
    let out = run_sh(&cmd)?;

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Launches command using `sh` shell
fn run_sh(cmd: &str) -> io::Result<Output> {
    Command::new("sh").arg("-c").arg(cmd).output()
}

/// Loads app launcher
fn build_app_launcher(dock: &Rc<GtkBox>, cfg: &ConfigSettings, dots_orientation: Orientation) {
    // Separator between apps and app launcher
    if cfg.show_separator == Toggle::On {
        let separator = Separator::new(dots_orientation);
        separator.set_widget_name("separator");
        dock.append(&separator);
    }

    // App launcher itself
    let launcher_icon = Image::from_icon_name(&cfg.app_launcher_icon);
    launcher_icon.set_pixel_size(32);

    let launcher_wrapper = GtkBox::new(dots_orientation, 0);
    launcher_wrapper.set_widget_name("app-launcher");
    launcher_wrapper.append(&launcher_icon);

    // Open app launcher using command specified in config file
    let cmd = cfg.app_launcher_command.clone();
    let launcher_gesture = GestureClick::builder().button(0).build();
    launcher_gesture.connect_pressed(move |_, n_press, _, _| {
        if n_press == 1 {
            if let Err(e) = Command::new("sh").arg("-c").arg(&cmd).spawn().and_then(|mut c| {
                thread::spawn(move || {
                    let _ = c.wait();
                });

                Ok(())
            }) {
                eprintln!("Failed to spawn app launcher: {e}");
            }
        }
    });
    launcher_wrapper.add_controller(launcher_gesture);
    dock.append(&launcher_wrapper);
}

/// Queries Hyprland for currently open clients using `hyprctl`, deserializes JSON output
fn fetch_hyprland_clients() -> Vec<HyprlandClient> {
    match Command::new("hyprctl").arg("clients").arg("-j").output() {
        Ok(output) => {
            serde_json::from_slice::<Vec<HyprlandClient>>(&output.stdout).unwrap_or_default()
        },
        Err(err) => {
            eprintln!("Failed to execute `hyprctl clients -j`: {err}");

            Vec::new()
        }
    }
}

/// Converts `dock_position` setting from configuration file into layout values
///
/// Falls back to default if the value is incorrect
///
/// # Returns
///
/// A tuple containing:
/// * `Edge`: Anchor for the dock and dock trigger
/// * `Orientation`: Orientation for the dock and dock trigger
/// * `Orientation`: Orientation for app dots, separator
fn parse_dock_position(cfg: &ConfigSettings) -> (Edge, Orientation, Orientation) {
    match cfg.dock_position.to_lowercase().as_str() {
        "left" => (Edge::Left, Orientation::Vertical, Orientation::Horizontal),
        "right" => (Edge::Right, Orientation::Vertical, Orientation::Horizontal),
        "top" => (Edge::Top, Orientation::Horizontal, Orientation::Vertical),
        _ => (Edge::Bottom, Orientation::Horizontal, Orientation::Vertical)    // "bottom" and any invalid value fallback here
    }
}

/// Loads configuration from `~/.config/hydock/config.toml`
///
/// Falls back to default settings if fails
fn load_config() -> ConfigSettings {
    let home = env::var("HOME").ok();
    let Some(home) = home else {
        eprintln!("$HOME is not set; using default configuration");

        return ConfigSettings::default();
    };
    let path = format!("{home}/.config/hydock/config.toml");

    if let Ok(toml_data) = fs::read_to_string(path) {
        match toml::from_str::<Config>(&toml_data) {
            Ok(config) => config.config,
            Err(err) => {
                eprintln!("Failed to parse config; using defaults: {err}");

                ConfigSettings::default()
            }
        }
    } else {
        ConfigSettings::default()
    }
}

/// Loads stylesheet from `~/.config/hydock/style.css`
fn load_style() {
    let Some(home) = env::var("HOME").ok() else {
        eprintln!("$HOME is not set; skipping style load");

        return;
    };
    let path = format!("{home}/.config/hydock/style.css");

    if let Ok(css_data) = fs::read_to_string(path) {
        let provider = CssProvider::new();
        provider.load_from_data(&css_data);

        if let Some(display) = Display::default() {
            style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_USER);
        } else {
            eprintln!("No GDK display; skipping style application");
        }
    }
}
