// Token claims structure
use serde::{Deserialize, Serialize};

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
