use actix_web::{web, App, Error, HttpRequest, HttpResponse};
use base64::decode;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use log::{error, info};
use nostr::{Event, Kind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::{
    handlers::handler::ErrorResponse,
    models::{
        jwt::LoginResponse,
        nostr::{NostrEvent, NostrNip98Event},
    },
    services::jwt_service::generate_token,
};

const SCHEME: &str = "Nostr";

pub async fn validate_nip98(req: HttpRequest) -> Result<Event, actix_web::Error> {
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
    let b_token = match decode(&token) {
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
        Kind => {
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

    let verified = ev.verify();

    match verified {
        Ok(verified) => return Ok(ev),
        Err(err) => {
            return Err(actix_web::error::ErrorInternalServerError(
                "Failed to verify event",
            ))
        }
    }
}
