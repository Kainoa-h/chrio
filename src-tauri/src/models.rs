use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{FromRow, SqlitePool};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, FromRow, Type)]
pub struct Client {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub dob: String,
    pub sex: String,
    pub registration_date: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct CreateClientDto {
    pub firstname: String,
    pub lastname: String,
    pub dob: String,
    pub sex: String,
}

impl Client {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Client,
            r#"SELECT id as "id: i32", firstname, lastname, dob, sex, registration_date FROM clients ORDER BY registration_date DESC"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, dto: CreateClientDto) -> Result<i32, sqlx::Error> {
        let registration_date = Utc::now().to_rfc3339();

        let id = sqlx::query!(
            "INSERT INTO clients (firstname, lastname, dob, sex, registration_date) VALUES (?, ?, ?, ?, ?)",
            dto.firstname,
            dto.lastname,
            dto.dob,
            dto.sex,
            registration_date
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(id as i32)
    }
}
