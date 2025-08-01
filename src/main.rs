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
use serde::Deserialize;
use glib::{
    ControlFlow,
    timeout_add_seconds_local
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct HyprClient {
    class: String
}

#[derive(Deserialize, Debug)]
struct Workspace {
    id: i32,
    name: String
}

fn main() {
    let app = Application::builder()
        .application_id("com.github.desyatkoff.hydock")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn fetch_hypr_clients() -> Vec<HyprClient> {
    let output = Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output()
        .expect("Failed to execute `hyprctl`");

    return serde_json::from_slice::<Vec<HyprClient>>(&output.stdout).unwrap_or_default();
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

    if let Ok(css_data) = fs::read_to_string(
        format!("{}/.config/hydock/style.css", std::env::var("HOME").unwrap())
    ) {
        let provider = CssProvider::new();
        provider.load_from_data(&css_data);

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }

    let container = Rc::new(GtkBox::new(gtk4::Orientation::Horizontal, 0));
    container.set_widget_name("dock");

    window.set_child(Some(&*container));
    window.show();

    let container_clone = Rc::clone(&container);

    timeout_add_seconds_local(1, move || {
        while let Some(child) = container_clone.first_child() {
            container_clone.remove(&child);
        }

        let mut counts: HashMap<String, usize> = HashMap::new();

        for client in fetch_hypr_clients() {
            *counts.entry(client.class.to_lowercase()).or_insert(0) += 1;
        }

        let mut entries: Vec<_> = counts.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (class, count) in entries {
            let app_icon = gtk4::Image::from_icon_name(&class);

            if app_icon.icon_name().is_none() {
                app_icon.set_icon_name(Some("application-x-executable"));
            }

            app_icon.set_pixel_size(32);

            let wrapper = GtkBox::new(gtk4::Orientation::Vertical, 0);
            wrapper.set_widget_name("app-icon");
            wrapper.append(&app_icon);

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
