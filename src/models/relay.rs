use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Relay {
    pub uuid: String,
    pub user_npub: String,
    pub name: String,
    pub description: String,
    pub subdomain: String,
    pub custom_domain: String,
    pub instance_type: String,
    pub instance_id: String,
    pub instance_ip: String,
    pub implementation: String,
    pub cloud_provider: String,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub expires_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl Relay {
    pub fn from_db_relay(relay: Relay) -> Self {
        Relay {
            uuid: relay.uuid,
            user_npub: relay.user_npub,
            name: relay.name,
            description: relay.description,
            subdomain: relay.subdomain,
            custom_domain: relay.custom_domain,
            instance_type: relay.instance_type,
            instance_id: relay.instance_id,
            instance_ip: relay.instance_ip,
            implementation: relay.implementation,
            cloud_provider: relay.cloud_provider,
            write_whitelist: relay.write_whitelist,
            read_whitelist: relay.read_whitelist,
            created_at: relay.created_at,
            updated_at: relay.updated_at,
            expires_at: relay.expires_at,
            deleted_at: relay.deleted_at,
        }
    }
}
