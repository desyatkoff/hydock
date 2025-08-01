use gtk4::prelude::*;
use gtk4::{
    Application,
    ApplicationWindow,
    Box as GtkBox,
    CssProvider,
    Label
};
use gtk4_layer_shell::{
    Edge,
    LayerShell
};
use std::{
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
    address: String,
    workspace: Workspace,
    class: String,
    title: String,
    initial_class: String,
    initial_title: String,
    pid: u32,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    id: i32,
    name: String,
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
        .default_height(40)
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
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let container = Rc::new(GtkBox::new(gtk4::Orientation::Horizontal, 8));

    container.set_widget_name("dock");

    window.set_child(Some(&*container));
    window.show();

    let container_clone = Rc::clone(&container);

    timeout_add_seconds_local(1, move || {
        while let Some(child) = container_clone.first_child() {
            container_clone.remove(&child);
        }

        for client in fetch_hypr_clients() {
            let label = Label::new(Some(&client.initial_class));

            container_clone.append(&label);
        }

        return ControlFlow::Continue;
    });
}
