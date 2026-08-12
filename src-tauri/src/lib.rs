mod ports;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
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
        .manage(ports::PortState::default())
        .invoke_handler(tauri::generate_handler![ports::list_ports])
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

            // Poll from Rust so the 2s cadence is immune to WebView2 throttling the
            // webview's timers when the widget is occluded. Skips entirely while
            // hidden in the tray (PRD §10 M2).
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let win = handle.get_webview_window("main");
                let visible = win
                    .as_ref()
                    .and_then(|w| w.is_visible().ok())
                    .unwrap_or(false);
                if !visible {
                    #[cfg(debug_assertions)]
                    eprintln!("[portpeek] tick skipped (hidden in tray)");
                    continue;
                }
                if let Some(state) = handle.try_state::<ports::PortState>() {
                    match ports::scan(&state) {
                        Ok(rows) => {
                            let _ = handle.emit("ports", rows);
                        }
                        Err(e) => {
                            let _ = handle.emit("ports-error", e);
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
