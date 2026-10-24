use crate::backup::BackupState;
use crate::db::DbState;
use crate::models::{
    Client, CreateClientDto, CreateSessionDto, Session, UpdateClientDto, UpdateSessionDto,
};
use base64::{engine::general_purpose, Engine as _};
use specta::specta;
use std::fs;
use tauri::{AppHandle, Manager, State};

fn validate_client_name(name: &str) -> Result<(), String> {
    if !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        Ok(())
    } else {
        Err("Client names must contain only letters A–Z, a–z, and numbers 0–9.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::validate_client_name;

    #[test]
    fn client_names_allow_only_nonempty_ascii_alphanumerics() {
        for name in ["Alice", "BOB", "Client123", "123"] {
            assert!(validate_client_name(name).is_ok(), "Rejected {name:?}");
        }
        for name in [
            "",
            "Anne Marie",
            "Anne-Marie",
            "O'Neil",
            "照片",
            "José",
            "a/b",
            "a\\b",
            "a:b",
            "a?b",
            "a*b",
            "a\"b",
            "a<b",
            "a>b",
            "a|b",
            "a.",
            "a ",
            "a\0b",
        ] {
            assert!(validate_client_name(name).is_err(), "Accepted {name:?}");
        }
    }
}

#[tauri::command]
#[specta]
pub async fn get_clients(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
) -> Result<Vec<Client>, String> {
    let _access = backup.access()?;
    Client::find_all(&*db).await.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn add_client(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    client: CreateClientDto,
) -> Result<i32, String> {
    let _access = backup.access()?;
    validate_client_name(&client.firstname)?;
    validate_client_name(&client.lastname)?;
    Client::create(&*db, client)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn update_client(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    client: UpdateClientDto,
) -> Result<(), String> {
    let _access = backup.access()?;
    validate_client_name(&client.firstname)?;
    validate_client_name(&client.lastname)?;
    Client::update(&*db, client)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn get_client_sessions(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    client_id: i32,
) -> Result<Vec<Session>, String> {
    let _access = backup.access()?;
    Session::find_by_client_id(&*db, client_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn get_session(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    id: i32,
) -> Result<Session, String> {
    let _access = backup.access()?;
    match Session::find_by_id(&*db, id)
        .await
        .map_err(|e| e.to_string())?
    {
        Some(s) => Ok(s),
        None => Err("Session not found".to_string()),
    }
}

#[tauri::command]
#[specta]
pub async fn add_session(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    session: CreateSessionDto,
) -> Result<i32, String> {
    let _access = backup.access()?;
    Session::create(&*db, session)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn update_session(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    session: UpdateSessionDto,
) -> Result<(), String> {
    let _access = backup.access()?;
    Session::update(&*db, session)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn save_image(
    backup: State<'_, BackupState>,
    app_handle: AppHandle,
    client_id: i32,
    client_firstname: String,
    session_no: i32,
    image_type: String,
    base64_image: String,
) -> Result<String, String> {
    let _access = backup.access()?;
    validate_client_name(&client_firstname)?;
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
pub async fn read_image_base64(
    backup: State<'_, BackupState>,
    path: String,
) -> Result<String, String> {
    let _access = backup.access()?;
    let image_data = fs::read(&path).map_err(|e| e.to_string())?;
    let base64_data = general_purpose::STANDARD.encode(image_data);
    Ok(format!("data:image/jpeg;base64,{}", base64_data))
}

#[tauri::command]
#[specta]
pub async fn get_next_session_number(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    client_id: i32,
) -> Result<i32, String> {
    let _access = backup.access()?;
    Session::get_next_session_number(&*db, client_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub async fn update_session_crop(
    backup: State<'_, BackupState>,
    db: State<'_, DbState>,
    session_id: i32,
    image_type: String,
    crop_data: String,
) -> Result<(), String> {
    let _access = backup.access()?;
    Session::update_crop(&*db, session_id, &image_type, crop_data)
        .await
        .map_err(|e| e.to_string())
}
