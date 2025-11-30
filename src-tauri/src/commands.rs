use crate::db::DbState;
use crate::models::{Client, CreateClientDto};
use specta::specta;
use tauri::State;

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
