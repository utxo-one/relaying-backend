use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use bech32::{FromBase32, ToBase32, Variant};
use std::error::Error;
use std::fmt;

pub async fn generate_random_string(n: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .map(char::from)
        .take(n)
        .collect()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataResponse<T> {
    pub data: T,
}

impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        DataResponse { data }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub struct Bech32Error(String);

// Implement std::error::Error for the custom error type
impl Error for Bech32Error {}

// Implement std::fmt::Display for the custom error type
impl fmt::Display for Bech32Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bech32Error: {}", self.0)
    }
}

// Implement the bech32_encode function with the custom error type
pub fn bech32_encode(hex_key: &String) -> Result<String, Bech32Error> {
    let hrp = "npub";
    let data = hex::decode(hex_key).map_err(|_| Bech32Error("Invalid key".to_string()))?;
    bech32::encode(&hrp, &data.to_base32(), Variant::Bech32)
        .map_err(|_| Bech32Error("Failed to encode key to bech32".to_string()))
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        ErrorResponse { error }
    }
}
