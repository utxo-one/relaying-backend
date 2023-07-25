use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use sqlx::PgPool;
use std::fmt;

use crate::{
    cloud_provider::{CloudProvider, InstanceType},
    relay::RelayImplementation,
    util::{DataResponse, ErrorResponse},
};

#[derive(Debug)]
pub enum RelayOrderRepositoryError {
    SqlxError(SqlxError),
    NotFound,
}

impl From<SqlxError> for RelayOrderRepositoryError {
    fn from(err: SqlxError) -> Self {
        RelayOrderRepositoryError::SqlxError(err)
    }
}

impl fmt::Display for RelayOrderRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RelayOrderRepositoryError::SqlxError(err) => err.fmt(f),
            RelayOrderRepositoryError::NotFound => write!(f, "Relay order not found"),
        }
    }
}

pub struct RelayOrderRepository {
    pub pool: PgPool,
}

impl RelayOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        relay_order: CreateRelayOrder,
    ) -> Result<RelayOrder, RelayOrderRepositoryError> {
        let uuid = uuid::Uuid::new_v4().to_string();
        let relay_order: RelayOrder = sqlx::query_as::<_, RelayOrder>(
            "
            INSERT INTO relay_orders (uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status)
            VALUES ($1, $2, $3, $4::relay_cloud_provider, $5::relay_instance_type, $6::relay_implementation, $7, $8::relay_order_status)
            RETURNING uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
            ")
            .bind(uuid)
            .bind(relay_order.user_npub)
            .bind(relay_order.amount)
            .bind(relay_order.cloud_provider.as_str())
            .bind(relay_order.instance_type.as_str())
            .bind(relay_order.implementation.as_str())
            .bind(relay_order.hostname)
            .bind(relay_order.status)
            .fetch_one(&self.pool)
            .await?;

        Ok(relay_order)
    }

    pub async fn get_one(&self, uuid: String) -> Result<RelayOrder, RelayOrderRepositoryError> {
        let relay_order: RelayOrder = sqlx::query_as::<_, RelayOrder>(
            "
            SELECT uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
            FROM relay_orders
            WHERE uuid = $1
            ")
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        Ok(relay_order)
    }

    pub async fn delete(&self, uuid: String) -> Result<(), RelayOrderRepositoryError> {
        sqlx::query(
            "
            DELETE FROM relay_orders
            WHERE uuid = $1
            ",
        )
        .bind(uuid)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

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

async fn create_relay_order_handler(
    pool: web::Data<PgPool>,
    relay_order_repo: web::Data<RelayOrderRepository>,
    data: web::Json<CreateRelayOrder>,
) -> impl Responder {
    let order = relay_order_repo.create(data.into_inner()).await;

    match order {
        Ok(order) => HttpResponse::Created().json(DataResponse::new(order)),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse::new(e.to_string())),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/relay_orders").route(web::post().to(create_relay_order_handler)));
}

#[cfg(test)]
mod tests {
    use super::RelayOrderRepository;
    use crate::relay_order::{
        create_relay_order_handler, CreateRelayOrder, RelayOrder, RelayOrderStatus,
    };
    use crate::{
        cloud_provider::{CloudProvider, InstanceType},
        relay::RelayImplementation,
        user::UserRepository,
        util::{generate_random_string, DataResponse},
    };
    use actix_web::{web::Data, App};
    use sqlx::PgPool;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url =
            dotenvy::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");
        pool
    }

    async fn create_test_user(pool: &PgPool) -> String {
        let user_npub = generate_random_string(16).await;

        let user = UserRepository::new(pool.clone())
            .create(&user_npub)
            .await
            .expect("Failed to create user");

        user.npub
    }

    async fn delete_test_relay_order(pool: &PgPool, uuid: String) {
        RelayOrderRepository::new(pool.clone())
            .delete(uuid)
            .await
            .expect("Failed to delete relay order");
    }

    async fn delete_test_user(pool: &PgPool, npub: String) {
        UserRepository::new(pool.clone())
            .delete(&npub)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    async fn test_create_and_delete_relay_order() {
        let pool = create_test_pool().await;
        let npub = create_test_user(&pool).await;

        let create = CreateRelayOrder {
            user_npub: npub.clone(),
            amount: 1,
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let repo = RelayOrderRepository::new(pool.clone());
        let relay_order = repo
            .create(create)
            .await
            .expect("Failed to create relay order");

        assert_eq!(relay_order.user_npub, npub);

        delete_test_user(&pool, npub).await;
        delete_test_relay_order(&pool, relay_order.uuid.clone()).await;

        repo.get_one(relay_order.uuid)
            .await
            .expect_err("Failed to delete relay order");
    }

    #[tokio::test]
    async fn test_get_relay_order() {
        let pool = create_test_pool().await;
        let npub = create_test_user(&pool).await;

        let create = CreateRelayOrder {
            user_npub: npub.clone(),
            amount: 1,
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let repo = RelayOrderRepository::new(pool.clone());

        let relay_order = repo
            .create(create)
            .await
            .expect("Failed to create relay order");

        let relay_order = repo
            .get_one(relay_order.uuid)
            .await
            .expect("Failed to get relay order");

        assert_eq!(relay_order.user_npub, npub);

        delete_test_user(&pool.clone(), npub).await;
        delete_test_relay_order(&pool, relay_order.uuid).await;
    }

    #[tokio::test]
    async fn test_handle_create_relay_order() {
        let pool = create_test_pool().await;
        let user_npub = create_test_user(&pool).await;
        let order: CreateRelayOrder = CreateRelayOrder {
            user_npub: user_npub.clone(),
            amount: 1000,
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let repo = RelayOrderRepository::new(pool.clone());

        let app = actix_web::test::init_service(
            App::new()
                .app_data(Data::new(pool.clone()))
                .app_data(Data::new(repo))
                .route(
                    "/relay_orders",
                    actix_web::web::post().to(create_relay_order_handler),
                ),
        )
        .await;
        let req = actix_web::test::TestRequest::post()
            .uri("/relay_orders")
            .set_json(&order)
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), 201);
        let response: DataResponse<RelayOrder> = actix_web::test::read_body_json(resp).await;
        assert_eq!(response.data.user_npub, user_npub);

        delete_test_user(&pool, user_npub).await;
        delete_test_relay_order(&pool, response.data.uuid).await;
    }
}