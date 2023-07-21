-- Add up migration script here
CREATE TABLE relays (
  uuid VARCHAR(50) NOT NULL UNIQUE PRIMARY KEY,
  user_npub VARCHAR(30) NOT NULL REFERENCES users(npub),
  name VARCHAR(30) NOT NULL,
  description TEXT NOT NULL,
  subdomain VARCHAR(30),
  custom_domain VARCHAR(100),
  instance_type VARCHAR(30) NOT NULL,
  instance_id VARCHAR(50) NOT NULL,
  instance_ip VARCHAR(50) NOT NULL,
  implementation VARCHAR(30) NOT NULL,
  cloud_provider VARCHAR(30) NOT NULL,
  write_whitelist JSONB NOT NULL,
  read_whitelist JSONB NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL,
  deleted_at TIMESTAMP
);