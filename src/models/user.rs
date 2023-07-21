use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub npub: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl User {
    pub fn from_db_user(db_user: User) -> Self {
        User {
            npub: db_user.npub,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
            deleted_at: db_user.deleted_at,
        }
    }
}
