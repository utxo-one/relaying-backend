-- Add up migration script here
CREATE TYPE relay_order_status AS ENUM (
    'pending', 'paid', 'redeemed', 'expired'
);

CREATE TYPE relay_cloud_provider AS ENUM (
    'aws', 'gcp', 'azure'
);

CREATE TYPE relay_implementation AS ENUM (
    'strfry', 'nostrrelayrs', 'nostream'
);

CREATE TYPE relay_instance_type AS ENUM (
    'awst2micro', 'awst2nano', 'awst2small', 'awst2medium', 'awst2large',
    'gcpn1standard1', 'gcpn1standard2', 'gcpn1standard4',
    'azureb1s', 'azureb1ms', 'azureb2s', 'azureb2ms'
);

CREATE TYPE relay_state AS ENUM (
    'rebooting', 'initializing', 'online', 'offline', 'deleted'
);

CREATE TABLE users (
  npub VARCHAR(100) NOT NULL UNIQUE PRIMARY KEY,
  hexpub VARCHAR(100) NOT NULL UNIQUE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP
);