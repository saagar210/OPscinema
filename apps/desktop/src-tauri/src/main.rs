use std::sync::Arc;

use opscinema_desktop_backend::api::{runtime_events::RuntimeEventBus, tauri_commands, Backend};
use opscinema_desktop_backend::storage::db::Storage;
use tauri::Manager;

fn main() {
    let run_result = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("failed to resolve app_data_dir: {e}"))?;
            let db_path = data_dir.join("state.sqlite");
            let assets_root = data_dir.join("assets");
            let storage = Storage::open(&db_path, &assets_root)
                .map_err(|e| format!("storage init failed: {e}"))?;

            let event_bus = RuntimeEventBus::with_app(app.handle().clone());
            let backend = Arc::new(Backend::new(storage));
            let event_bus_for_hook = event_bus.clone();
            backend.set_capture_status_hook(Some(Arc::new(move |status| {
                let _ = event_bus_for_hook.emit_capture_status(&status);
            })));

            app.manage(backend);
            app.manage(event_bus);
            Ok(())
        })
        .invoke_handler(tauri_commands::invoke_handler())
        .run(tauri::generate_context!());

    if let Err(err) = run_result {
        eprintln!("tauri runtime failed: {err}");
        std::process::exit(1);
    }
}
