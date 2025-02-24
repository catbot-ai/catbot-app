use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, LogicalSize, Manager, WebviewUrl,
};

pub fn handle_tray_event(app_handle: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Right,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let maybe_window = app_handle.get_webview_window("main");
        let window = match maybe_window {
            Some(window) => window,
            None => tauri::WebviewWindowBuilder::new(
                app_handle,
                "Jupiter",
                WebviewUrl::External("https://jup.ag".parse().unwrap()),
            )
            .title("Jupiter")
            .always_on_top(true)
            .build()
            .unwrap(),
        };

        let _ = window.set_size(LogicalSize::new(640, 480));
        let _ = window.show();
        let _ = window.set_focus();
    }
}
