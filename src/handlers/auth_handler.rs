use actix_web::{web, HttpRequest, HttpResponse, Error};
use crate::{models::jwt::LoginResponse, services::{jwt_service::generate_token, nostr_service::validate_nip98}};


async fn auth_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    
    match validate_nip98(req).await {
        Ok(ev) => {
            match generate_token(&ev.pubkey).await {
                Ok(token) => {
                    return Ok(HttpResponse::Ok().json(LoginResponse { token }))
                },
                Err(e) => {
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Failed to generate token",
                    ))
                }
            }
        },
        Err(e) => {
            return Err(actix_web::error::ErrorUnauthorized(
                "Failed to validate NIP98",
            ))
        }
    }


}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(auth_handler)));
}