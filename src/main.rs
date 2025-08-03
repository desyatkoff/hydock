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

use gtk4::prelude::*;
use gtk4::{
    Application,
    ApplicationWindow,
    Box as GtkBox,
    CssProvider
};
use gtk4_layer_shell::{
    Edge,
    LayerShell
};
use std::{
    collections::HashMap,
    fs,
    process::Command,
    rc::Rc
};
use serde::{
    Deserialize,
    Serialize
};
use glib::{
    ControlFlow,
    timeout_add_seconds_local
};

#[derive(Deserialize, Debug)]
struct HyprlandClient {
    class: String
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    config: ConfigSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConfigSettings {
    chaos_mode: bool,
    exclusive_zone: i32,
    pinned_applications: Vec<String>
}

impl Default for ConfigSettings {
    fn default() -> Self {
        ConfigSettings {
            chaos_mode: false.into(),
            exclusive_zone: 66.into(),
            pinned_applications: Vec::new().into(),
        }
    }
}

fn main() {
    let app = Application::builder()
        .application_id("com.github.desyatkoff.hydock")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn load_config() -> ConfigSettings {
    if let Ok(toml_data) = fs::read_to_string(format!(
        "{}/.config/hydock/config.toml",
        std::env::var("HOME").unwrap()
    )) {
        match toml::from_str::<Config>(&toml_data) {
            Ok(config) => config.config,
            Err(_) => ConfigSettings::default()
        }
    } else {
        return ConfigSettings::default();
    }
}

fn fetch_hyprland_clients() -> Vec<HyprlandClient> {
    let output = Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output()
        .expect("Failed to execute `hyprctl`");

    return serde_json::from_slice::<Vec<HyprlandClient>>(&output.stdout).unwrap_or_default();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .title("Hydock")
        .can_focus(false)
        .build();
    window.init_layer_shell();
    window.set_anchor(Edge::Bottom, true);
    window.set_layer(gtk4_layer_shell::Layer::Top);

    let container = Rc::new(GtkBox::new(gtk4::Orientation::Horizontal, 0));
    container.set_widget_name("dock");

    window.set_child(Some(&*container));
    window.show();

    let container_clone = Rc::clone(&container);

    timeout_add_seconds_local(1, move || {
            window.set_exclusive_zone(load_config().exclusive_zone);

        if let Ok(css_data) = fs::read_to_string(format!(
            "{}/.config/hydock/style.css",
            std::env::var("HOME").unwrap()
        )) {
            let provider = CssProvider::new();
            provider.load_from_data(&css_data);

            gtk4::style_context_add_provider_for_display(
                &gtk4::gdk::Display::default().unwrap(),
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
            );
        }

        while let Some(child) = container_clone.first_child() {
            container_clone.remove(&child);
        }

        let mut counts: HashMap<String, usize> = HashMap::new();

        for pinned in load_config().pinned_applications {
            *counts.entry(pinned.to_lowercase()).or_insert(0) += 0;
        }

        for client in fetch_hyprland_clients() {
            *counts.entry(client.class.to_lowercase()).or_insert(0) += 1;
        }

        let mut entries: Vec<_> = counts.into_iter().collect();

        if !load_config().chaos_mode {
            entries.sort_by(|a, b| a.0.cmp(&b.0));
        }

        for (class, count) in entries {
            let app_icon = gtk4::Image::from_icon_name(&class);

            if app_icon.icon_name().is_none() {
                app_icon.set_icon_name(Some("application-x-executable"));
            }

            app_icon.set_pixel_size(32);

            let wrapper = GtkBox::new(gtk4::Orientation::Vertical, 0);
            wrapper.set_widget_name("app-icon");
            wrapper.append(&app_icon);

            let gesture = gtk4::GestureClick::builder().button(0).build();

            gesture.connect_pressed(move |_, n_press, _, _| {
                if n_press == 1 {
                    let address_cmd_str = format!(
                        "hyprctl clients -j | jq -r '[.[] | select(.class == \"{}\")][0].address'",
                        class
                    );
                    let address_output = Command::new("sh")
                        .arg("-c")
                        .arg(address_cmd_str)
                        .output()
                        .expect("Failed to execute `hyprctl clients`");

                    let address_str = String::from_utf8_lossy(&address_output.stdout).trim().to_string();

                    let focus_cmd_str = format!("hyprctl dispatch focuswindow address:{}", address_str);
                    let focus_output = Command::new("sh")
                        .arg("-c")
                        .arg(&focus_cmd_str)
                        .output()
                        .expect("Failed to execute `hyprctl dispatch focuswindow`");

                    let focus_str = String::from_utf8_lossy(&focus_output.stdout).trim().to_string();

                    if focus_str == "No such window found" {
                        let _ = Command::new(format!("/usr/bin/{}", class))
                            .spawn()
                            .unwrap();
                    }
                }
            });

            wrapper.add_controller(gesture);

            let app_dots_box = GtkBox::new(gtk4::Orientation::Horizontal, 4);
            app_dots_box.set_widget_name("app-dots-box");
            app_dots_box.set_halign(gtk4::Align::Center);

            for _ in 0..count {
                let app_dot = GtkBox::new(gtk4::Orientation::Vertical, 0);
                app_dot.set_widget_name("app-dot");
                app_dot.set_size_request(4, 4);

                app_dots_box.append(&app_dot);
            }

            wrapper.append(&app_dots_box);

            container_clone.append(&wrapper);
        }

        return ControlFlow::Continue;
    });
}
