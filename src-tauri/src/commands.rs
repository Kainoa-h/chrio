use crate::db::DbState;
use crate::models::{Client, CreateClientDto, Session, CreateSessionDto};
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
