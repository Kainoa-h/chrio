mod commands;
mod db;
mod models;

use tauri_specta::{collect_commands, Builder};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::get_clients,
            commands::add_client,
            commands::get_client_sessions,
            commands::add_session
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let db = db::init_db(&handle).await.expect("Failed to initialize database");
                handle.manage(db);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
