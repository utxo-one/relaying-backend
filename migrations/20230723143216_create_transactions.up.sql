-- Add up migration script here
CREATE TABLE transactions (
    uuid varchar(50) NOT NULL UNIQUE PRIMARY KEY,
    user_npub varchar(30) NOT NULL references users(npub),
    relay_order_uuid varchar(50) NOT NULL references relay_orders(uuid),
    amount int NOT NULL,
    type varchar(12) NOT NULL,
    status varchar(12) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
