mod camera;
mod commands;
mod db;
mod models;

use std::sync::Arc;

use tauri_specta::{collect_commands, Builder};
use tauri::Manager;
use tokio::sync::watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
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
            commands::list_cameras,
            commands::start_camera,
            commands::stop_camera,
            commands::snap_photo
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let db = db::init_db(&handle).await.expect("Failed to initialize database");
                handle.manage(db);

                // Initialize camera state and MJPEG server
                let (frame_tx, frame_rx) = watch::channel(Arc::new(Vec::new()));
                let port = camera::start_stream_server(frame_rx.clone()).await;
                let camera_state = camera::CameraState {
                    frame_tx,
                    frame_rx,
                    camera_handle: std::sync::Mutex::new(None),
                    stream_port: port,
                };
                handle.manage(camera_state);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
