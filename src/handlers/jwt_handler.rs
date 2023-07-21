use crate::models::jwt::{LoginInfo, LoginResponse};
use crate::services::jwt_service::generate_token;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;

pub async fn login_handler(info: web::Json<LoginInfo>) -> Result<HttpResponse, Error> {
    // Assuming "npub" is a unique identifier for the user (e.g., username or email)
    let npub = &info.npub;

    // Generate a JWT token based on the "npub"
    match generate_token(npub).await {
        Ok(token) => {
            Ok(HttpResponse::Ok().json(LoginResponse { token }))
        },
        Err(e) => {
            eprintln!("Error generating token: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError(
                "Failed to generate token",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::handlers::jwt_handler::configure_routes;
    use crate::models::jwt::LoginInfo;
    use crate::models::jwt::LoginResponse;
    use actix_web::{http::StatusCode, test, App};

    #[tokio::test]
    async fn test_it_can_get_jwt_token() {
        let mut app = test::init_service(App::new().configure(configure_routes)).await;

        let login_info = LoginInfo {
            npub: "test".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&login_info)
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let login_response: LoginResponse = serde_json::from_slice(&body).unwrap();

        assert!(login_response.token.len() > 0);
    }
}
