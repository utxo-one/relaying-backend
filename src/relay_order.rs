use actix_web::HttpRequest;
use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use sqlx::Error as SqlxError;
use sqlx::PgPool;
use std::fmt;

use crate::middleware::AuthorizationService;
use crate::relay_order;
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

#[derive(Clone)]
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

    pub async fn get_one(&self, uuid: &String) -> Result<RelayOrder, RelayOrderRepositoryError> {
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

    pub async fn get_all(&self) -> Result<Vec<RelayOrder>, RelayOrderRepositoryError> {
        let relay_orders: Vec<RelayOrder> = sqlx::query_as::<_, RelayOrder>(
            "
            SELECT uuid, user_npub, amount, cloud_provider, instance_type, implementation, hostname, status, created_at, updated_at
            FROM relay_orders
            ",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(relay_orders)
    }

    pub async fn delete(&self, uuid: &String) -> Result<(), RelayOrderRepositoryError> {
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

    pub async fn update_status(
        &self,
        uuid: &str,
        status: RelayOrderStatus,
    ) -> Result<(), RelayOrderRepositoryError> {
        let query = sqlx::query(
            "
            UPDATE relay_orders
            SET status = $1::relay_order_status
            WHERE uuid = $2
            ",
        )
        .bind(&status)
        .bind(&uuid)
        .execute(&self.pool)
        .await?;

        eprintln!("Query: {:?}", query);
        eprintln!("UUID: {}, Status: {}", &uuid, &status.to_string());

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
    _auth: AuthorizationService,
    relay_order_repo: web::Data<RelayOrderRepository>,
    data: web::Json<CreateRelayOrder>,
) -> impl Responder {
    let order = relay_order_repo.create(data.into_inner()).await;

    match order {
        Ok(order) => HttpResponse::Created().json(DataResponse::new(order)),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse::new(e.to_string())),
    }
}

#[derive(Debug, Serialize)]
struct JsonResponse {
    message: String,
}

pub async fn nodeless_webhook_handler(
    req: HttpRequest,
    relay_order_repo: web::Data<RelayOrderRepository>,
    payload: web::Bytes,
) -> impl Responder {
    let payload: Value = match serde_json::from_slice(&payload) {
        Ok(payload) => payload,
        Err(err) => {
            eprintln!("Could not parse the JSON payload from Nodeless: {}", err);
            return HttpResponse::InternalServerError().body("Could not parse JSON payload");
        }
    };

    let headers = req.headers();
    let sig = headers
        .get("nodeless-signature")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    let secret = dotenvy::var("NODELESS_WEBHOOK_SECRET").unwrap().to_string();

    let json_bytes = serde_json::to_vec(&payload).expect("Failed to serialize payload to JSON");

    let expected_signature = calculate_hmac_sha256(&json_bytes, &secret);

    if sig != &expected_signature {
        eprintln!(
            "Invalid Signature. Expected: {}, Received: {}, Payload: {}",
            expected_signature, sig, payload
        );
        return HttpResponse::Unauthorized().body("HMAC signature verification failed");
    }

    let status = payload["status"].as_str().unwrap_or_default();
    if status == "paid" || status == "overpaid" {
        eprintln!(
            "Webhook received successfully. Order uuid: {}.",
            payload["metadata"]["order_uuid"].to_string()
        );
        let order = relay_order_repo
            .update_status(
                payload["metadata"]["order_uuid"].as_str().unwrap(),
                RelayOrderStatus::Paid,
            )
            .await;

        match order {
            Ok(_) => HttpResponse::Ok().body("Order status updated successfully"),
            Err(e) => {
                eprintln!("Failed to update relay order status: {}", e);
                HttpResponse::InternalServerError().body("Failed to update relay order status")
            }
        }
    } else {
        eprintln!("Invalid Nodeless payment notification message received - status is not paid or overpaid.");
        HttpResponse::BadRequest().body("Invalid Nodeless payment notification message received - status is not paid or overpaid.")
    }
}

fn calculate_hmac_sha256(payload: &[u8], secret: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload);
    let result = mac.finalize();
    let bytes = result.into_bytes();
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/relay_orders").route(web::post().to(create_relay_order_handler)))
        .service(
            web::resource("/nodeless_webhook").route(web::post().to(nodeless_webhook_handler)),
        );
}

#[cfg(test)]
mod tests {
    use super::RelayOrderRepository;
    use crate::auth::generate_jwt_by_hex;
    use crate::relay;
    use crate::relay_order::{
        create_relay_order_handler, nodeless_webhook_handler, CreateRelayOrder, RelayOrder,
        RelayOrderStatus,
    };
    use crate::util::TestUtils;
    use crate::{
        cloud_provider::{CloudProvider, InstanceType},
        relay::RelayImplementation,
        user::UserRepository,
        util::{generate_random_string, DataResponse},
    };
    use actix_web::{web::Data, App};
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_calculate_hmac_sha256() {
        let payload = "test".as_bytes();
        let secret = "test";
        let expected_hash = "88cd2108b5347d973cf39cdf9053d7dd42704876d8c9a9bd8e2d168259d3ddf7";

        let hash = super::calculate_hmac_sha256(payload, secret);

        assert_eq!(hash, expected_hash);
    }

    #[tokio::test]
    async fn test_create_get_delete_relay_order() {
        let test_utils = TestUtils::new().await;
        let npub = test_utils.create_user().await.npub;

        let create = CreateRelayOrder {
            user_npub: npub.clone(),
            amount: 1,
            cloud_provider: CloudProvider::AWS,
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending,
        };

        let repo = test_utils.relay_order_repo.clone();
        let relay_order = repo
            .create(create)
            .await
            .expect("Failed to create relay order");

        assert_eq!(relay_order.user_npub, npub);

        let relay_order = repo
            .get_one(&relay_order.uuid)
            .await
            .expect("Failed to get relay order");

        assert_eq!(relay_order.user_npub, npub);

        repo.delete(&relay_order.uuid).await;

        repo.get_one(&relay_order.uuid)
            .await
            .expect_err("Failed to delete relay order");

        test_utils.revert_migrations().await;
    }

    #[tokio::test]
    async fn test_handle_create_relay_order() {
        let test_utils = TestUtils::new().await;
        let user = test_utils.create_user().await;
        let order = test_utils.create_relay_order(&user.npub.as_str());

        let repo = RelayOrderRepository::new(test_utils.pool.clone());
        let jwt_token = generate_jwt_by_hex(user.hexpub.as_str()).unwrap();
        let app = actix_web::test::init_service(
            App::new()
                .app_data(Data::new(test_utils.pool.clone()))
                .app_data(Data::new(repo))
                .route(
                    "/relay_orders",
                    actix_web::web::post().to(create_relay_order_handler),
                ),
        )
        .await;
        let req = actix_web::test::TestRequest::post()
            .uri("/relay_orders")
            .insert_header(("Authorization", jwt_token))
            .set_json(&order.await)
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), 201);
        let response: DataResponse<RelayOrder> = actix_web::test::read_body_json(resp).await;
        assert_eq!(response.data.user_npub, user.npub);

        test_utils.revert_migrations().await;
    }
}
