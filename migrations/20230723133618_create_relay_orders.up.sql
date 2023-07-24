-- Add up migration script here
CREATE TABLE relay_orders (
    uuid varchar(50) NOT NULL UNIQUE PRIMARY KEY,
    user_npub varchar(30) NOT NULL references users(npub),
    amount int NOT NULL,
    cloud_provider varchar(30) NOT NULL,
    instance_type varchar(30) NOT NULL,
    implementation varchar(30) NOT NULL,
    hostname varchar(255) NOT NULL,
    status varchar(12) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);