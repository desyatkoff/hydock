use gtk4::prelude::*;
use gtk4::{
    Application,
    ApplicationWindow,
    CssProvider
};
use gtk4_layer_shell::{
    Edge,
    LayerShell
};
use std::fs;

fn main() {
    let app = Application::builder()
        .application_id("com.github.desyatkoff.hydock")
        .build();

    app.connect_activate(build_ui);
    app.run();
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
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }

    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    container.set_widget_name("dock");

    container.append(&gtk4::Label::new(Some("Placeholder")));
    container.append(&gtk4::Label::new(Some("Placeholder")));
    container.append(&gtk4::Label::new(Some("Placeholder")));

    window.set_child(Some(&container));
    window.show();
}
