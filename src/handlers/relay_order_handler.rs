use crate::{
    handlers::handler::ErrorResponse,
    repositories::{self, relay_order_repository::CreateRelayOrder},
};
use actix_web::http::header::ContentType;
use actix_web::test::TestRequest;
use actix_web::{test, web, App, HttpResponse, Responder};
use sqlx::PgPool;

use super::handler::DataResponse;

async fn create_relay_order_handler(
    pool: web::Data<PgPool>,
    data: web::Json<CreateRelayOrder>,
) -> impl Responder {
    let order =
        repositories::relay_order_repository::create_relay_order(data.into_inner(), &pool).await;

    match order {
        Ok(order) => HttpResponse::Created().json(DataResponse::new(order)),
        Err(e) => {
            println!("Failed to create relay order: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::new(
                e.to_string(),
            ))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/relay_orders").route(web::post().to(create_relay_order_handler)));
}

#[cfg(test)]
mod tests {
    use actix_web::web::Data;

    use crate::{
        models::{relay_orders::{RelayOrder, RelayOrderStatus}, user::User},
        repositories::{
            relay_order_repository::{create_relay_order, delete_relay_order},
            user_repository::{create_user, delete_user},
        },
        util::generators::generate_random_string,
    };

    use super::*;

    async fn create_test_pool() -> PgPool {
        dotenvy::dotenv().ok();

        let db_url =
            dotenvy::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let pool = PgPool::connect(&db_url)
            .await
            .expect("Failed to create test pool");
        pool
    }

    async fn create_test_user() -> String {
        let user_npub = generate_random_string(16).await;
        let pool = create_test_pool().await;
        let user = create_user(&pool, &user_npub)
            .await
            .expect("Failed to create user");

        user.npub
    }

    async fn delete_test_user(npub: &str) {
        let pool = create_test_pool().await;
        delete_user(&pool, &npub)
            .await
            .expect("Failed to delete user");
    }

    async fn delete_test_relay_order(uuid: String) {
        let pool = create_test_pool().await;
        delete_relay_order(&pool, uuid)
            .await
            .expect("Failed to delete relay order");
    }

    #[tokio::test]
    async fn test_handle_create_relay_order() {
        let user_npub = create_test_user().await;
        let pool = create_test_pool().await;

        let order: CreateRelayOrder = CreateRelayOrder {
            user_npub: user_npub.clone(),
            amount: 1000,
            cloud_provider: "aws".to_string(),
            instance_type: "t2.micro".to_string(),
            implementation: "openvpn".to_string(),
            hostname: "test".to_string(),
            status: RelayOrderStatus::Pending.to_string(),
        };

        let app = test::init_service(App::new().app_data(Data::new(pool.clone())).route("/relay_orders", web::post().to(create_relay_order_handler))).await;
        let req = test::TestRequest::post().uri("/relay_orders").set_json(&order).to_request();
        let resp = test::call_service(&app, req).await;
        
        assert_eq!(resp.status(), 201);
        let response: DataResponse<RelayOrder> = test::read_body_json(resp).await;
        assert_eq!(response.data.user_npub, user_npub);

        delete_test_user(&user_npub).await;
        delete_test_relay_order(response.data.uuid).await;
    }
}
