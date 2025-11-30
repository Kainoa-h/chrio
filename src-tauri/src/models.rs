use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;

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
