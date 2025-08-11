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
    ControlFlow,
    timeout_add_seconds_local
};
use gtk4::{
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
    style_context_add_provider_for_display,
    prelude::*
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
    env,
    fs
};
use std::collections::HashMap;
use std::process::Command;
use std::rc::Rc;

/// Wrapper for the full Hydock configuration
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    config: ConfigSettings
}

/// Config settings loaded from `config.toml`
///
/// * `app_launcher_command`: Shell command to execute when the app launcher is clicked
/// * `auto_hide`: Hide dock when unfocused
/// * `chaos_mode`: Enable random order of app icons
/// * `ignore_applications`: List of application class names that should never appear in the dock
/// * `pinned_applications`: List of application class names that should always appear in the dock
/// * `show_app_launcher`: Add app launcher button on the right
/// * `show_separator`: Add separator between apps and app launcher
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigSettings {
    app_launcher_command: String,
    auto_hide: bool,
    chaos_mode: bool,
    ignore_applications: Vec<String>,
    pinned_applications: Vec<String>,
    show_app_launcher: bool,
    show_separator: bool
}

/// Implements default config settings
impl Default for ConfigSettings {
    fn default() -> Self {
        ConfigSettings {
            app_launcher_command: "rofi -show drun".into(),
            auto_hide: false.into(),
            chaos_mode: false.into(),
            ignore_applications: Vec::new().into(),
            pinned_applications: Vec::new().into(),
            show_app_launcher: true.into(),
            show_separator: true.into()
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
    hydock.set_anchor(Edge::Bottom, true);
    hydock.set_layer(Layer::Top);
    hydock.auto_exclusive_zone_enable();

    // Trigger for showing dock again after it became hidden (when `auto_hide` is `true`)
    let trigger = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .title("Hydock Trigger")
        .can_focus(false)
        .default_width(2147483647)
        .default_height(1)
        .build();
    trigger.init_layer_shell();
    trigger.set_anchor(Edge::Bottom, true);
    trigger.set_layer(Layer::Top);
    trigger.show();

    // Dock panel itself
    let dock = Rc::new(GtkBox::new(
        Orientation::Horizontal,
        0
    ));
    let dock_clone = Rc::clone(&dock);
    dock.set_widget_name("dock");
    hydock.set_child(Some(&*dock));

    // Main loop for refreshing dock
    timeout_add_seconds_local(1, move || {
        let hydock_clone = hydock.clone();
        let hydock_motion = EventControllerMotion::new();
        hydock_motion.connect_leave(move |_| {
            hydock_clone.hide();
        });

        let hydock_clone = hydock.clone();
        let trigger_motion = EventControllerMotion::new();
        trigger_motion.connect_enter(move |_, _, _| {
            hydock_clone.show();
        });

        if load_config().auto_hide == true {
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

        while let Some(child) = dock_clone.first_child() {
            dock_clone.remove(&child);
        }

        build_apps(&dock_clone);

        if load_config().show_app_launcher == true {
            build_app_launcher(&dock_clone);
        }

        return ControlFlow::Continue;
    });
}

/// Loads app icons & dots
fn build_apps(dock: &Rc<GtkBox>) {
    let mut counts: HashMap<String, usize> = HashMap::new();

    // Add actually opened apps
    for client in fetch_hyprland_clients() {
        *counts.entry(client.class.to_lowercase()).or_insert(0) += 1;
    }

    // Ensure pinned apps appear in dock even if they have no open windows
    for pinned in load_config().pinned_applications {
        *counts.entry(pinned.to_lowercase()).or_insert(0) += 0;
    }

    // Remove unwanted apps
    for ignored in load_config().ignore_applications {
        counts.remove_entry(&ignored.to_lowercase());
    }

    // Collect apps into a Vector
    let mut entries: Vec<(String, usize)> = counts.into_iter().collect();

    // Sort app icons in alphabetical order if `chaos_mode` is `false`
    if !load_config().chaos_mode {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    // Add app icons & dots
    for (class, count) in entries {
        // Icons lookup
        let app_icon = Image::from_icon_name(&class);
        app_icon.set_pixel_size(32);

        if app_icon.icon_name().is_none() {
            app_icon.set_icon_name(Some("application-default-icon"));
        }

        let apps_wrapper = GtkBox::new(Orientation::Vertical, 0);
        apps_wrapper.set_widget_name("app-icon");
        apps_wrapper.append(&app_icon);

        // Try to focus the first window of the clicked app class using `hyprctl`
        // If it fails (e.g., no such window), fallback to launching the app binary from `/usr/bin/` directory
        let apps_gesture = GestureClick::builder().button(0).build();
        apps_gesture.connect_pressed(move |_, n_press, _, _| {
            if n_press == 1 {
                let address_cmd_str = format!(
                    "hyprctl clients -j | jq -r '[.[] | select(.class == \"{}\")][0].address'",    // Get address of the first client with specified class
                    class
                );
                let address_output = Command::new("sh")
                    .arg("-c")
                    .arg(address_cmd_str)
                    .output()
                    .expect(&format!(
                        "Failed to execute `hyprctl clients -j | jq -r '[.[] | select(.class == \"{}\")][0].address'`",
                        class
                    ));
                let address_str = String::from_utf8_lossy(&address_output.stdout).trim().to_string();

                let focus_cmd_str = format!("hyprctl dispatch focuswindow address:{}", address_str);
                let focus_output = Command::new("sh")
                    .arg("-c")
                    .arg(&focus_cmd_str)
                    .output()
                    .expect(&format!(
                        "Failed to execute `hyprctl dispatch focuswindow address:{}`",
                        address_str
                    ));
                let focus_str = String::from_utf8_lossy(&focus_output.stdout).trim().to_string();

                if focus_str == "No such window found" {
                    let _ = Command::new(format!("/usr/bin/{}", class))
                        .spawn()
                        .unwrap();
                }
            }
        });
        apps_wrapper.add_controller(apps_gesture);

        // Represent app's window count using dots
        let app_dots_box = GtkBox::new(Orientation::Horizontal, 4);
        app_dots_box.set_widget_name("app-dots-box");
        app_dots_box.set_halign(Align::Center);

        for _ in 0..count {
            let app_dot = GtkBox::new(Orientation::Vertical, 0);
            app_dot.set_widget_name("app-dot");
            app_dot.set_size_request(4, 4);

            app_dots_box.append(&app_dot);
        }

        apps_wrapper.append(&app_dots_box);
        dock.append(&apps_wrapper);
    }
}

/// Loads app launcher
fn build_app_launcher(dock: &Rc<GtkBox>) {
    // Separator between apps and app launcher
    if load_config().show_separator == true {
        let separator = Separator::new(Orientation::Vertical);
        separator.set_widget_name("separator");
        dock.append(&separator);
    }

    // App launcher itself
    let launcher_icon = Image::from_icon_name("applications-all-symbolic");
    launcher_icon.set_pixel_size(32);

    let launcher_wrapper = GtkBox::new(Orientation::Vertical, 0);
    launcher_wrapper.set_widget_name("app-launcher");
    launcher_wrapper.append(&launcher_icon);

    // Open app launcher using command specified in config file
    let launcher_gesture = GestureClick::builder().button(0).build();
    launcher_gesture.connect_pressed(move |_, n_press, _, _| {
        if n_press == 1 {
            let _ = Command::new("sh")
                .arg("-c")
                .arg(load_config().app_launcher_command)
                .spawn();
        }
    });
    launcher_wrapper.add_controller(launcher_gesture);
    dock.append(&launcher_wrapper);
}

/// Queries Hyprland for currently open clients using `hyprctl`, deserializes JSON output
fn fetch_hyprland_clients() -> Vec<HyprlandClient> {
    let output = Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output()
        .expect("Failed to execute `hyprctl clients -j`");

    return serde_json::from_slice::<Vec<HyprlandClient>>(&output.stdout).unwrap_or_default();
}

/// Loads configuration from `~/.config/hydock/config.toml`, returns default settings if fails
fn load_config() -> ConfigSettings {
    if let Ok(toml_data) = fs::read_to_string(format!(
        "{}/.config/hydock/config.toml",
        env::var("HOME").unwrap()
    )) {
        match toml::from_str::<Config>(&toml_data) {
            Ok(config) => config.config,
            Err(_) => ConfigSettings::default()
        }
    } else {
        return ConfigSettings::default();
    }
}

/// Loads stylesheet from `~/.config/hydock/style.css`
fn load_style() {
    if let Ok(css_data) = fs::read_to_string(format!(
        "{}/.config/hydock/style.css",
        env::var("HOME").unwrap()
    )) {
        let provider = CssProvider::new();
        provider.load_from_data(&css_data);

        style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &provider,
            STYLE_PROVIDER_PRIORITY_USER
        );
    }
}
