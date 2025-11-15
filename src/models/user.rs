use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    SuperAdmin,
    SchoolAdmin,
    Parent,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub school_id: Option<i32>,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub nik: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub reset_password_token: Option<String>,
    pub reset_password_expires: Option<NaiveDateTime>,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn get_role(&self) -> UserRole {
        match self.role.as_str() {
            "super_admin" => UserRole::SuperAdmin,
            "school_admin" => UserRole::SchoolAdmin,
            "parent" => UserRole::Parent,
            _ => UserRole::Parent,
        }
    }
}
