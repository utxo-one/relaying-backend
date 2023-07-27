use crate::util::ErrorResponse;
use actix_web::{web, Error, FromRequest, HttpRequest, HttpResponse};
use base64::{engine::general_purpose, Engine};
use chrono::{TimeZone, Utc};
use futures::future::{ready, Ready};
use jsonwebtoken::{encode, EncodingKey, Header};
use nostr::{Event, Kind};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Models & DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInfo {
    pub npub: String, // Assuming "npub" is a string identifier like username or email
}

// LoginResponse represents the JSON response containing the token
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

const SCHEME: &str = "Nostr";

// -----------------------------------------------------------------------------
// Functions
// -----------------------------------------------------------------------------

pub fn generate_token(sub: &secp256k1::XOnlyPublicKey) -> Result<String, Error> {
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

pub fn generate_jwt_by_hex(sub: &str) -> Result<String, Error> {
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

pub fn validate_nip98(req: &HttpRequest) -> Result<Event, actix_web::Error> {
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_none() {
        return Err(actix_web::error::ErrorInternalServerError(
            "Failed to generate token",
        ));
    }

    let auth = auth_header.unwrap().to_str().unwrap().trim().to_string();
    if !auth.starts_with(SCHEME) {
        return Err(actix_web::error::ErrorInternalServerError(
            "No scheme defined",
        ));
    }

    let token = auth[SCHEME.len()..].trim().to_string();
    println!("the unecoded token is: {}", token);
    let b_token = match general_purpose::STANDARD.decode(&token) {
        Ok(token) => token,
        Err(error) => {
            println!("Failed to decode token. token: {}", error);

            return Err(actix_web::error::ErrorBadRequest(serde_json::json!(
                ErrorResponse {
                    error: "invalid token".to_string(),
                }
            )));
        }
    };

    if b_token.is_empty() || b_token[0] != b'{' {
        return Err(actix_web::error::ErrorInternalServerError(
            "Failed to generate token",
        ));
    }

    let ev: nostr::Event = match serde_json::from_slice(&b_token) {
        Ok(ev) => ev,
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(
                "invalid nostr event. err: ".to_string() + &err.to_string(),
            ))
        }
    };

    match ev.kind {
        Kind::Ephemeral(27_235) => {
            println!("the event is a nip98 event");
        }
        _ => {
            return Err(actix_web::error::ErrorInternalServerError(
                "wrong nostr kind",
            ))
        }
    }

    let created_at_utc = match Utc.timestamp_opt(ev.created_at.as_i64(), 0) {
        chrono::LocalResult::Single(time) => time,
        chrono::LocalResult::None => {
            return Err(actix_web::error::ErrorInternalServerError(
                "Invalid timestamp",
            ));
        }
        chrono::LocalResult::Ambiguous(_, _) => {
            return Err(actix_web::error::ErrorInternalServerError(
                "Ambiguous timestamp encountered",
            ));
        }
    };

    let diff_time = Utc::now()
        .signed_duration_since(created_at_utc)
        .num_seconds();

    if diff_time.abs() > 10 {
        return Err(actix_web::error::ErrorInternalServerError(
            "timestamp out of range",
        ));
    }

    Ok(ev)
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

async fn auth_handler(Nip98PubKey(pubkey): Nip98PubKey) -> Result<HttpResponse, Error> {
    match generate_token(&pubkey) {
        Ok(token) => Ok(HttpResponse::Ok().json(LoginResponse { token })),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(auth_handler)));
}

// -----------------------------------------------------------------------------
// Extractor
// -----------------------------------------------------------------------------
pub struct Nip98PubKey(XOnlyPublicKey);

impl FromRequest for Nip98PubKey {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        ready(validate_nip98(req).map(|e| Nip98PubKey(e.pubkey)))
    }
}
