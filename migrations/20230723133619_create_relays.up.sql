-- Add up migration script here
CREATE TABLE relays (
  uuid VARCHAR(50) NOT NULL UNIQUE PRIMARY KEY,
  user_npub VARCHAR(30) NOT NULL REFERENCES users(npub),
  relay_order_uuid VARCHAR(50) NOT NULL REFERENCES relay_orders(uuid),
  name VARCHAR(30) NOT NULL,
  description TEXT NOT NULL,
  subdomain VARCHAR(30),
  custom_domain VARCHAR(100),
  instance_type relay_instance_type NOT NULL,
  instance_id VARCHAR(50) NOT NULL,
  instance_ip VARCHAR(50) NOT NULL,
  implementation relay_implementation NOT NULL,
  cloud_provider relay_cloud_provider NOT NULL,
  write_whitelist JSONB NOT NULL,
  read_whitelist JSONB NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL,
  deleted_at TIMESTAMP
);