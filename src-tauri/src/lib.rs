use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_window_state::StateFlags;

/// Bring the widget back from the tray. `unminimize` matters as well as `show`:
/// hiding and minimising are independent states, and a widget that was minimised
/// before being hidden stays off-screen if you only call `show`.
fn show_widget(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::default()
                // Position + size only. Restoring VISIBLE would let a run that ended
                // hidden-in-tray come back invisible with no window to click.
                .with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
                .build(),
        )
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "Show PortPeek", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit PortPeek", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("PortPeek")
                .menu(&menu)
                // Left click restores; the menu is reserved for right click, otherwise
                // the common case (get my widget back) costs two clicks.
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_widget(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_widget(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
