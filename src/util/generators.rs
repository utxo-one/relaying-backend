use rand::distributions::Alphanumeric;
use rand::Rng;

pub async fn generate_random_string(n: usize) -> String {
    let mut rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .map(char::from)
        .take(n)
        .collect()
}
