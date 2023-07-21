use actix_cors::Cors;

pub async fn cors_middleware() -> Cors {
    Cors::default()
        .allow_any_header()
        .allow_any_method()
        .allow_any_origin()
}
