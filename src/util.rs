use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub async fn generate_random_string(n: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .map(char::from)
        .take(n)
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct DataResponse<T> {
    pub data: T,
}

impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        DataResponse { data }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        ErrorResponse { error }
    }
}
