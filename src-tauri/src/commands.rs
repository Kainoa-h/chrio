use crate::db::DbState;
use crate::models::{Client, CreateClientDto, Session, CreateSessionDto};
use specta::specta;
use tauri::{State, AppHandle, Manager};
use std::fs;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};

#[tauri::command]
#[specta]
pub async fn get_clients(db: State<'_, DbState>) -> Result<Vec<Client>, String> {
    Client::find_all(&*db)
        .await
        .map_err(|e| e.to_string())
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
pub async fn get_client_sessions(db: State<'_, DbState>, client_id: i32) -> Result<Vec<Session>, String> {
    Session::find_by_client_id(&*db, client_id)
        .await
        .map_err(|e| e.to_string())
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
pub async fn save_image(
    app_handle: AppHandle,
    client_id: i32,
    client_firstname: String,
    session_no: i32,
    image_type: String,
    base64_image: String
) -> Result<String, String> {
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
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
pub async fn get_next_session_number(db: State<'_, DbState>, client_id: i32) -> Result<i32, String> {
    Session::get_next_session_number(&*db, client_id)
        .await
        .map_err(|e| e.to_string())
}
