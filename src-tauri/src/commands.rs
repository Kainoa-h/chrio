use crate::db::DbState;
use crate::models::{Client, CreateClientDto, CreateSessionDto, Session, UpdateClientDto, UpdateSessionDto};
use base64::{engine::general_purpose, Engine as _};
use specta::specta;
use std::fs;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
#[specta]
pub async fn get_clients(db: State<'_, DbState>) -> Result<Vec<Client>, String> {
    Client::find_all(&*db).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn add_client(db: State<'_, DbState>, client: CreateClientDto) -> Result<i32, String> {
    Client::create(&*db, client)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn update_client(db: State<'_, DbState>, client: UpdateClientDto) -> Result<(), String> {
    Client::update(&*db, client)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn get_client_sessions(
    db: State<'_, DbState>,
    client_id: i32,
) -> Result<Vec<Session>, String> {
    Session::find_by_client_id(&*db, client_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn get_session(db: State<'_, DbState>, id: i32) -> Result<Session, String> {
    match Session::find_by_id(&*db, id).await.map_err(|e| e.to_string())? {
        Some(s) => Ok(s),
        None => Err("Session not found".to_string()),
    }
}

#[tauri::command]
#[specta]
pub async fn add_session(db: State<'_, DbState>, session: CreateSessionDto) -> Result<i32, String> {
    Session::create(&*db, session)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn update_session(
    db: State<'_, DbState>,
    session: UpdateSessionDto,
) -> Result<(), String> {
    Session::update(&*db, session)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn save_image(
    app_handle: AppHandle,
    client_id: i32,
    client_firstname: String,
    session_no: i32,
    image_type: String,
    base64_image: String,
) -> Result<String, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let photos_dir = app_data_dir.join("photos");

    let client_dir_name = format!("{}_{}", client_id, client_firstname);
    let session_dir_name = format!("session_{}", session_no);

    let target_dir = photos_dir.join(&client_dir_name).join(&session_dir_name);

    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let file_name = format!("{}_{}.jpg", image_type, client_firstname);
    let file_path = target_dir.join(&file_name);

    // Remove header if present (e.g., "data:image/jpeg;base64,")
    let base64_data = base64_image.split(",").last().unwrap_or(&base64_image);

    let image_data = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| e.to_string())?;

    fs::write(&file_path, image_data).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
#[specta]
pub async fn read_image_base64(path: String) -> Result<String, String> {
    let image_data = fs::read(&path).map_err(|e| e.to_string())?;
    let base64_data = general_purpose::STANDARD.encode(image_data);
    Ok(format!("data:image/jpeg;base64,{}", base64_data))
}

#[tauri::command]
#[specta]
pub async fn get_next_session_number(
    db: State<'_, DbState>,
    client_id: i32,
) -> Result<i32, String> {
    Session::get_next_session_number(&*db, client_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn update_session_crop(
    db: State<'_, DbState>,
    session_id: i32,
    image_type: String,
    crop_data: String,
) -> Result<(), String> {
    Session::update_crop(&*db, session_id, &image_type, crop_data)
        .await
        .map_err(|e| e.to_string())
}
