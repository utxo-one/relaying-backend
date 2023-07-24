use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::{cloud_provider::{CloudProvider, InstanceType}, relay::{RelayImplementation, Relay}};

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "relay_order_status", rename_all = "lowercase")]
pub enum RelayOrderStatus {
    Pending,
    Paid,
    Redeemed,
}

impl RelayOrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelayOrderStatus::Pending => "pending",
            RelayOrderStatus::Paid => "paid",
            RelayOrderStatus::Redeemed => "redeemed",
        }
    }
}


impl ToString for RelayOrderStatus {
    fn to_string(&self) -> String {
        match &self {
            RelayOrderStatus::Pending => "pending".to_string(),
            RelayOrderStatus::Paid => "paid".to_string(),
            RelayOrderStatus::Redeemed => "redeemed".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct RelayOrder {
    pub uuid: String,
    pub user_npub: String,
    pub amount: i32,
    pub cloud_provider: CloudProvider,
    pub instance_type: InstanceType,
    pub implementation: RelayImplementation,
    pub hostname: String,
    pub status: RelayOrderStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl RelayOrder {
    pub fn from_db_relay_order(relay_order: RelayOrder) -> Self {
        RelayOrder {
            uuid: relay_order.uuid,
            user_npub: relay_order.user_npub,
            amount: relay_order.amount,
            cloud_provider: relay_order.cloud_provider,
            instance_type: relay_order.instance_type,
            implementation: relay_order.implementation,
            hostname: relay_order.hostname,
            status: relay_order.status,
            created_at: relay_order.created_at,
            updated_at: relay_order.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateRelayOrder {
    pub user_npub: String,
    pub amount: i32,
    pub cloud_provider: CloudProvider,
    pub instance_type: InstanceType,
    pub implementation: RelayImplementation,
    pub hostname: String,
    pub status: RelayOrderStatus,
}
