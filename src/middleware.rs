use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

use crate::auth::Claims;

async fn jwt_middleware(
    req: actix_web::HttpRequest,
) -> Result<TokenData<Claims>, actix_web::Error> {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_header) = auth_header {
        let secret = dotenvy::var("JWT_SECRET").unwrap();
        let token = auth_header.to_str().unwrap_or("");
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        match decode::<Claims>(token, &decoding_key, &Validation::default()) {
            Ok(token_data) => Ok(token_data),
            Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
        }
    } else {
        Err(actix_web::error::ErrorUnauthorized("Missing token"))
    }
}
