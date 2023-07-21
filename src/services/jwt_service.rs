use crate::models::jwt::Claims;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use jsonwebtoken::{encode, EncodingKey, Header};

pub async fn generate_token(sub: &secp256k1::XOnlyPublicKey) -> Result<String, Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .expect("Could not set expiration time.");

    let claims = Claims {
        sub: sub.to_string(),
        exp: expiration.timestamp() as usize,
    };

    let secret = dotenvy::var("JWT_SECRET").unwrap();
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to generate token"))
}
