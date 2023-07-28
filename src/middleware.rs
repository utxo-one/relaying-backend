use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;

use crate::auth::Claims;

pub struct AuthorizationService {
    hexpub: Option<String>,
    npub: Option<String>,
}

impl AuthorizationService {
    pub fn hexpub(&self) -> Option<&String> {
        self.hexpub.as_ref()
    }

    pub fn npub(&self) -> Option<&String> {
        self.npub.as_ref()
    }
}

impl FromRequest for AuthorizationService {
    type Error = Error;
    type Future = Ready<Result<AuthorizationService, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header) = auth_header {
            let secret = dotenvy::var("JWT_SECRET").unwrap();
            let token = auth_header.to_str().unwrap_or("");
            let decoding_key = DecodingKey::from_secret(secret.as_ref());

            match decode::<Claims>(token, &decoding_key, &Validation::default()) {
                Ok(token_data) => {
                    // Extract the 'sub' field from the token and store it in AuthorizationService
                    let hexpub = token_data.claims.hexpub;
                    let npub = token_data.claims.npub;
                    let auth_service = AuthorizationService { hexpub: Some(hexpub), npub: Some(npub) };
                    ready(Ok(auth_service))
                }
                Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
            }
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Missing token")))
        }
    }
}
