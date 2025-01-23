use dioxus::{
    desktop::trayicon::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
    prelude::*,
};

fn main() {
    dioxus::launch(App);

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("foo".to_string()),
                copyright: Some("Copyright bar".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);

    let tray_icon = None;

    let tray_icon = dioxus::desktop::trayicon::init_tray_icon(tray_menu, tray_icon);
    tray_icon.set_title(Some("Hello, World!"));
}

#[component]
fn App() -> Element {
    rsx! { "HotDog!" }
}
