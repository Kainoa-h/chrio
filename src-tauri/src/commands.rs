use crate::db::DbState;
use crate::models::{Client, CreateClientDto};
use specta::specta;
use tauri::State;

#[tauri::command]
#[specta]
pub async fn get_clients(db: State<'_, DbState>) -> Result<Vec<Client>, String> {
    let clients = sqlx::query_as::<_, Client>(
        "SELECT * FROM clients ORDER BY registration_date DESC"
    )
    .fetch_all(&*db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(clients)
}

#[tauri::command]
#[specta]
pub async fn add_client(db: State<'_, DbState>, client: CreateClientDto) -> Result<i32, String> {
    let registration_date = chrono::Utc::now().to_rfc3339();
    
    let id = sqlx::query(
        "INSERT INTO clients (firstname, lastname, dob, sex, registration_date) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&client.firstname)
    .bind(&client.lastname)
    .bind(&client.dob)
    .bind(&client.sex)
    .bind(&registration_date)
    .execute(&*db)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    Ok(id as i32)
}
