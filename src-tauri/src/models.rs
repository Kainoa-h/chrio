use chrono::Local;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{FromRow, Row, SqlitePool};

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

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct UpdateClientDto {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub dob: String,
    pub sex: String,
}

impl Client {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Client>(
            r#"SELECT id, firstname, lastname, dob, sex, registration_date FROM clients ORDER BY registration_date DESC"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, dto: CreateClientDto) -> Result<i32, sqlx::Error> {
        let registration_date = Local::now().to_rfc3339();

        let id = sqlx::query(
            "INSERT INTO clients (firstname, lastname, dob, sex, registration_date) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(dto.firstname)
        .bind(dto.lastname)
        .bind(dto.dob)
        .bind(dto.sex)
        .bind(registration_date)
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(id as i32)
    }

    pub async fn update(pool: &SqlitePool, dto: UpdateClientDto) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE clients SET firstname = ?, lastname = ?, dob = ?, sex = ? WHERE id = ?"
        )
        .bind(dto.firstname)
        .bind(dto.lastname)
        .bind(dto.dob)
        .bind(dto.sex)
        .bind(dto.id)
        .execute(pool)
        .await?;

        Ok(())
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

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct UpdateSessionDto {
    pub id: i32,
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
        sqlx::query_as::<_, Session>(
            r#"SELECT 
                id, 
                client_id, 
                datetime, 
                session_number, 
                height, 
                weight, 
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
            ORDER BY session_number DESC"#
        )
        .bind(client_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"SELECT 
                id, 
                client_id, 
                datetime, 
                session_number, 
                height, 
                weight, 
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
            WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, dto: CreateSessionDto) -> Result<i32, sqlx::Error> {
        let datetime = Local::now().to_rfc3339();

        let next_session_number = sqlx::query(
            "SELECT MAX(session_number) as max_session FROM sessions WHERE client_id = ?"
        )
        .bind(dto.client_id)
        .fetch_one(pool)
        .await?
        .get::<Option<i32>, _>("max_session")
        .map(|n| n + 1)
        .unwrap_or(1);

        let id = sqlx::query(
            r#"INSERT INTO sessions (
                client_id, datetime, session_number, height, weight, 
                anterior, posterior, right_lateral, left_lateral, notes,
                anterior_crop, posterior_crop, right_lateral_crop, left_lateral_crop
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(dto.client_id)
        .bind(datetime)
        .bind(next_session_number)
        .bind(dto.height)
        .bind(dto.weight)
        .bind(dto.anterior)
        .bind(dto.posterior)
        .bind(dto.right_lateral)
        .bind(dto.left_lateral)
        .bind(dto.notes)
        .bind(dto.anterior_crop)
        .bind(dto.posterior_crop)
        .bind(dto.right_lateral_crop)
        .bind(dto.left_lateral_crop)
        .execute(pool)
        .await?
        .last_insert_rowid();

        Ok(id as i32)
    }

    pub async fn get_next_session_number(
        pool: &SqlitePool,
        client_id: i32,
    ) -> Result<i32, sqlx::Error> {
        let next_session_number: i32 = sqlx::query(
            "SELECT MAX(session_number) as max_session FROM sessions WHERE client_id = ?"
        )
        .bind(client_id)
        .fetch_one(pool)
        .await?
        .get::<Option<i32>, _>("max_session")
        .map(|n| n + 1)
        .unwrap_or(1)
        .try_into()
        .expect("how tf does a client have sm sessions");

        Ok(next_session_number)
    }

    pub async fn update_crop(
        pool: &SqlitePool,
        session_id: i32,
        image_type: &str,
        crop_data: String,
    ) -> Result<(), sqlx::Error> {
        match image_type {
            "anterior" => {
                sqlx::query(
                    "UPDATE sessions SET anterior_crop = ? WHERE id = ?"
                )
                .bind(crop_data)
                .bind(session_id)
                .execute(pool)
                .await?;
            }
            "posterior" => {
                sqlx::query(
                    "UPDATE sessions SET posterior_crop = ? WHERE id = ?"
                )
                .bind(crop_data)
                .bind(session_id)
                .execute(pool)
                .await?;
            }
            "right_lateral" => {
                sqlx::query(
                    "UPDATE sessions SET right_lateral_crop = ? WHERE id = ?"
                )
                .bind(crop_data)
                .bind(session_id)
                .execute(pool)
                .await?;
            }
            "left_lateral" => {
                sqlx::query(
                    "UPDATE sessions SET left_lateral_crop = ? WHERE id = ?"
                )
                .bind(crop_data)
                .bind(session_id)
                .execute(pool)
                .await?;
            }
            _ => return Err(sqlx::Error::RowNotFound),
        }
        Ok(())
    }

    pub async fn update(pool: &SqlitePool, dto: UpdateSessionDto) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE sessions SET 
                height = ?, weight = ?, 
                anterior = ?, posterior = ?, right_lateral = ?, left_lateral = ?, 
                notes = ?,
                anterior_crop = ?, posterior_crop = ?, right_lateral_crop = ?, left_lateral_crop = ?
            WHERE id = ?"#
        )
        .bind(dto.height)
        .bind(dto.weight)
        .bind(dto.anterior)
        .bind(dto.posterior)
        .bind(dto.right_lateral)
        .bind(dto.left_lateral)
        .bind(dto.notes)
        .bind(dto.anterior_crop)
        .bind(dto.posterior_crop)
        .bind(dto.right_lateral_crop)
        .bind(dto.left_lateral_crop)
        .bind(dto.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
