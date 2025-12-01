use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{FromRow, SqlitePool};

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

#[derive(Debug, Serialize, Deserialize, FromRow, Type)]
pub struct Session {
    pub id: i32,
    pub client_id: i32,
    pub datetime: String,
    pub session_number: i32,
    pub height: Option<f64>,
    pub weight: Option<f64>,
    pub anterior: Option<String>,
    pub posterior: Option<String>,
    pub right_lateral: Option<String>,
    pub left_lateral: Option<String>,
    pub notes: Option<String>,
    pub anterior_crop: Option<String>,
    pub posterior_crop: Option<String>,
    pub right_lateral_crop: Option<String>,
    pub left_lateral_crop: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct CreateSessionDto {
    pub client_id: i32,
    pub height: Option<f64>,
    pub weight: Option<f64>,
    pub anterior: Option<String>,
    pub posterior: Option<String>,
    pub right_lateral: Option<String>,
    pub left_lateral: Option<String>,
    pub notes: Option<String>,
    pub anterior_crop: Option<String>,
    pub posterior_crop: Option<String>,
    pub right_lateral_crop: Option<String>,
    pub left_lateral_crop: Option<String>,
}

impl Session {
    pub async fn find_by_client_id(
        pool: &SqlitePool,
        client_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Session,
            r#"SELECT 
                id as "id: i32", 
                client_id as "client_id: i32", 
                CAST(datetime AS TEXT) as datetime, 
                session_number as "session_number: i32", 
                height as "height: f64", 
                weight as "weight: f64", 
                anterior, 
                posterior, 
                right_lateral, 
                left_lateral, 
                notes,
                anterior_crop,
                posterior_crop,
                right_lateral_crop,
                left_lateral_crop
            FROM sessions 
            WHERE client_id = ? 
            ORDER BY session_number DESC"#,
            client_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, dto: CreateSessionDto) -> Result<i32, sqlx::Error> {
        let datetime = Utc::now().to_rfc3339();

        // Calculate the next session number for this client
        let next_session_number = sqlx::query!(
            "SELECT MAX(session_number) as max_session FROM sessions WHERE client_id = ?",
            dto.client_id
        )
        .fetch_one(pool)
        .await?
        .max_session
        .map(|n| n + 1)
        .unwrap_or(1);

        let id = sqlx::query!(
            r#"INSERT INTO sessions (
                client_id, datetime, session_number, height, weight, 
                anterior, posterior, right_lateral, left_lateral, notes,
                anterior_crop, posterior_crop, right_lateral_crop, left_lateral_crop
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            dto.client_id,
            datetime,
            next_session_number,
            dto.height,
            dto.weight,
            dto.anterior,
            dto.posterior,
            dto.right_lateral,
            dto.left_lateral,
            dto.notes,
            dto.anterior_crop,
            dto.posterior_crop,
            dto.right_lateral_crop,
            dto.left_lateral_crop
        )
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(id as i32)
    }

    pub async fn get_next_session_number(
        pool: &SqlitePool,
        client_id: i32,
    ) -> Result<i32, sqlx::Error> {
        let next_session_number: i32 = sqlx::query!(
            "SELECT MAX(session_number) as max_session FROM sessions WHERE client_id = ?",
            client_id
        )
        .fetch_one(pool)
        .await?
        .max_session
        .map(|n| n + 1)
        .unwrap_or(1)
        .try_into()
        .expect("how tf does a client have sm sessions");

        Ok(next_session_number)
    }
}
