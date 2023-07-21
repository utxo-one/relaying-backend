use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::models::jwt::Claims;

async fn validate_token(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
) -> Result<actix_web::HttpRequest, actix_web::Error> {
    // Get the Authorization header
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_header) = auth_header {
        let secret = dotenvy::var("JWT_SECRET").unwrap();
        let token = auth_header.to_str().unwrap_or("");
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        match decode::<Claims>(token, &decoding_key, &Validation::default()) {
            Ok(token_data) => {
                // Token is valid
                // You can now use token_data.claims.sub to identify the user
                Ok(req)
            },
            Err(_) => {
                // Token is invalid
                Err(actix_web::error::ErrorUnauthorized("Invalid token"))
            },
        }
    } else {
        // Token not provided
        Err(actix_web::error::ErrorUnauthorized("Missing token"))
    }
}