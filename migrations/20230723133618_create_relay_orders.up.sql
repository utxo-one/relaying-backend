CREATE TABLE relay_orders (
    uuid varchar(50) NOT NULL UNIQUE PRIMARY KEY,
    user_npub varchar(100) NOT NULL references users(npub),
    amount int NOT NULL,
    cloud_provider relay_cloud_provider NOT NULL,
    instance_type relay_instance_type NOT NULL,
    implementation relay_implementation NOT NULL,
    hostname varchar(255) NOT NULL,
    status relay_order_status NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);