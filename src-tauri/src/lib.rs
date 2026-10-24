mod backup;
mod commands;
mod db;
mod models;

use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

fn command_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::get_clients,
        commands::add_client,
        commands::update_client,
        commands::get_client_sessions,
        commands::get_session,
        commands::add_session,
        commands::update_session,
        commands::save_image,
        commands::read_image_base64,
        commands::get_next_session_number,
        commands::update_session_crop,
        backup::begin_backup,
        backup::end_backup,
        backup::export_backup,
        backup::validate_backup,
        backup::import_backup,
        backup::backup_startup_message
    ])
}

pub fn export_bindings() {
    command_builder()
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = command_builder();

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let root = handle
                    .path()
                    .app_data_dir()
                    .expect("Cannot locate app data");
                std::fs::create_dir_all(&root).expect("Cannot create app data directory");
                let data_lock = backup::lock_data_directory(&root)
                    .await
                    .expect("Cannot safely open Chrio data");
                handle.manage(data_lock);
                let startup_message = backup::recover_restore(&root)
                    .await
                    .expect("Cannot safely recover client data; recovery files have been retained");
                handle.manage(backup::BackupState::with_message(startup_message));
                let db = db::init_db(&handle)
                    .await
                    .expect("Failed to initialize database");
                handle.manage(db);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
