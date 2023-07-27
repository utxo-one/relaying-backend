use crate::{
    cloud_provider::{CloudProvider, InstanceType},
    relay::{CreateRelay, Relay, RelayImplementation, RelayRepository},
    relay_order::{self, CreateRelayOrder, RelayOrder, RelayOrderRepository, RelayOrderStatus},
    user::{User, UserRepository},
};
use bech32::{FromBase32, ToBase32, Variant};
use nostr::{prelude::ToBech32, Keys};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::error::Error;
use std::fmt;

pub async fn generate_random_string(n: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .map(char::from)
        .take(n)
        .collect()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataResponse<T> {
    pub data: T,
}

impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        DataResponse { data }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub struct Bech32Error(String);

// Implement std::error::Error for the custom error type
impl Error for Bech32Error {}

// Implement std::fmt::Display for the custom error type
impl fmt::Display for Bech32Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bech32Error: {}", self.0)
    }
}

// Implement the bech32_encode function with the custom error type
pub fn bech32_encode(hex_key: &String) -> Result<String, Bech32Error> {
    let hrp = "npub";
    let data = hex::decode(hex_key).map_err(|_| Bech32Error("Invalid key".to_string()))?;
    bech32::encode(&hrp, &data.to_base32(), Variant::Bech32)
        .map_err(|_| Bech32Error("Failed to encode key to bech32".to_string()))
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        ErrorResponse { error }
    }
}

#[derive(Clone)]
pub struct TestUtils {
    pub pool: PgPool,
    pub user_repo: UserRepository,
    pub relay_repo: RelayRepository,
    pub relay_order_repo: RelayOrderRepository,
}

impl TestUtils {
    pub async fn new() -> Self {
        let db_url =
            dotenvy::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");

        let user_repo = UserRepository::new(pool.clone());
        let relay_repo = RelayRepository::new(pool.clone());
        let relay_order_repo = RelayOrderRepository::new(pool.clone());

        Self {
            pool,
            user_repo,
            relay_repo,
            relay_order_repo,
        }
    }

    pub async fn create_user(&self) -> User {
        let keys = Keys::generate();
        let npub = keys.public_key().to_bech32().unwrap();
        let hexpub = keys.public_key().to_string();
        let user = self
            .user_repo
            .create(&npub, hexpub.clone().as_str())
            .await
            .unwrap();

        user
    }

    pub async fn delete_user(&self, npub: &str) {
        self.user_repo.delete(npub).await.unwrap();
    }

    pub async fn create_relay_order(&self, npub: &str) -> RelayOrder {
        let order = CreateRelayOrder {
            user_npub: npub.to_string(),
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            amount: 1000,
            implementation: RelayImplementation::Strfry,
            hostname: "test.relaying.io".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let order = self.relay_order_repo.create(order).await.unwrap();

        order
    }

    pub async fn create_relay(&self, order: RelayOrder) -> Relay {
        // Create a relay to update
        let relay = CreateRelay {
            user_npub: order.user_npub,
            relay_order_uuid: order.uuid,
            name: "test relay".to_string(),
            description: "test description".to_string(),
            subdomain: generate_random_string(10).await,
            custom_domain: generate_random_string(10).await,
            instance_type: InstanceType::AwsT2Nano,
            instance_id: generate_random_string(10).await,
            instance_ip: generate_random_string(10).await,
            implementation: RelayImplementation::Strfry,
            cloud_provider: CloudProvider::AWS,
            write_whitelist: json!({"key": "value"}),
            read_whitelist: json!({"key": "value"}),
            expires_at: chrono::Local::now().naive_utc(),
        };

        let relay = self.relay_repo.create(relay).await.unwrap();

        relay
    }

    pub async fn revert_migrations(self: &Self) -> Result<(), sqlx::Error> {
        let drop_query = "
            DROP TABLE IF EXISTS relay_orders CASCADE;
            DROP TABLE IF EXISTS relays CASCADE;
            DROP TABLE IF EXISTS users CASCADE;
    ";

        let _ = sqlx::query(drop_query).execute(&self.pool).await;

        Ok(())
    }

    pub async fn run_migrations(self: &Self) {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .unwrap();
    }
}
