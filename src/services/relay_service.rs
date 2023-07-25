use sqlx::PgPool;

use crate::{
    models::cloud_provider::LaunchCloudInstance,
    models::{
        cloud_provider::{CloudProvider, InstanceType},
        relay::{CreateRelay, Relay, RelayImplementation},
    },
    repositories::{relay_repository::RelayRepository, user_repository::UserRepository},
    services::aws_service::launch_instance,
    util::generators::generate_random_string,
};

pub struct CreateRelayService {
    pub user_npub: String,
    pub relay_order_uuid: String,
    pub name: String,
    pub description: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub instance_type: InstanceType,
    pub implementation: RelayImplementation,
    pub cloud_provider: CloudProvider,
    pub write_whitelist: serde_json::Value,
    pub read_whitelist: serde_json::Value,
    pub expires_at: chrono::NaiveDateTime,
}

pub async fn create_relay_service(
    pool: &PgPool,
    relay: CreateRelayService,
) -> Result<Relay, String> {
    let repo = UserRepository::new(&pool);
    if !repo.user_exists(relay.user_npub.clone()).await {
        return Err("User does not exist".to_string());
    }

    let launch = LaunchCloudInstance {
        name: relay.name.clone(),
        image_id: dotenvy::var("STRFRY_AMI").unwrap(),
        instance_type: relay.instance_type,
        implementation: relay.implementation,
    };

    let instance = launch_instance(launch).await;

    match instance {
        Ok(instance) => {
            let create_relay = CreateRelay {
                user_npub: relay.user_npub,
                relay_order_uuid: relay.relay_order_uuid,
                name: relay.name,
                description: relay.description,
                subdomain: relay.subdomain.unwrap_or_default(),
                custom_domain: relay.custom_domain.unwrap_or_default(),
                instance_type: relay.instance_type,
                instance_id: instance.id,
                instance_ip: instance.ip_address,
                implementation: relay.implementation,
                cloud_provider: relay.cloud_provider,
                write_whitelist: relay.write_whitelist,
                read_whitelist: relay.read_whitelist,
                expires_at: relay.expires_at,
            };

            let relay = RelayRepository::new(&pool)
                .create(create_relay)
                .await
                .expect("Failed to create relay");

            Ok(relay)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::{
        models::{
            cloud_provider::{CloudProvider, InstanceType},
            relay::RelayImplementation,
            relay_orders::{CreateRelayOrder, RelayOrderStatus},
        },
        repositories::{
            self, relay_order_repository::RelayOrderRepository, user_repository::UserRepository,
        },
        services::aws_service::terminate_instance,
    };

    use super::*;
    use serde_json::json;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url =
            dotenvy::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");

        pool
    }

    #[tokio::test]
    pub async fn test_create_relay_service() {
        let pool = create_test_pool().await;
        let npub = generate_random_string(16).await;
        let user = UserRepository::new(&pool).create(&npub).await;

        let order = RelayOrderRepository::new(&pool)
            .create(CreateRelayOrder {
                user_npub: user.unwrap().npub.clone(),
                amount: 100,
                cloud_provider: CloudProvider::AWS,
                instance_type: InstanceType::AwsT2Nano,
                implementation: RelayImplementation::Strfry,
                hostname: "test.relaying.io".to_string(),
                status: RelayOrderStatus::Pending,
            })
            .await
            .expect("Failed to create relay order");

        let user_npub = &npub;
        let name = "Test Relay".to_string();
        let description = "This is a test relay".to_string();
        let instance_type = InstanceType::AwsT2Nano;
        let implementation = RelayImplementation::Strfry;
        let cloud_provider = CloudProvider::AWS;
        let write_whitelist = json!({"key": "value"});
        let read_whitelist = json!({"key": "value"});
        let expires_at = chrono::Local::now().naive_utc();

        let create_relay = CreateRelayService {
            user_npub: user_npub.clone(),
            relay_order_uuid: order.uuid.clone(),
            name: name.clone(),
            description: description.clone(),
            subdomain: None,
            custom_domain: None,
            instance_type: instance_type,
            implementation: implementation,
            cloud_provider: cloud_provider,
            write_whitelist: write_whitelist.clone(),
            read_whitelist: read_whitelist.clone(),
            expires_at: expires_at.clone(),
        };

        let relay = create_relay_service(&pool, create_relay)
            .await
            .expect("Failed to create relay");

        assert_eq!(relay.name, name);
        assert_eq!(relay.description, description);
        assert_eq!(relay.write_whitelist, write_whitelist);
        assert_eq!(relay.read_whitelist, read_whitelist);

        let terminate_instance = terminate_instance(&relay.instance_id).await;
        assert!(terminate_instance.is_ok());

        let delete_relay = RelayRepository::new(&pool).delete(relay.uuid).await;
        assert!(delete_relay.is_ok());

        let delete_user = UserRepository::new(&pool).delete(&user_npub).await;
        assert!(delete_user.is_ok());
    }
}
