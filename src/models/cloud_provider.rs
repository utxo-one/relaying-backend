use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::relay::RelayImplementation;

pub struct CloudInstance {
    pub id: String,
    pub ip_address: String,
}

pub struct LaunchCloudInstance {
    pub name: String,
    pub image_id: String,
    pub instance_type: InstanceType,
    pub implementation: RelayImplementation,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "relay_cloud_provider", rename_all = "lowercase")]
pub enum CloudProvider {
    AWS,
    GCP,
    Azure,
}

impl CloudProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            CloudProvider::AWS => "aws",
            CloudProvider::GCP => "gcp",
            CloudProvider::Azure => "azure",
        }
    }
}

impl TryFrom<&str> for CloudProvider {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "aws" => Ok(CloudProvider::AWS),
            "gcp" => Ok(CloudProvider::GCP),
            "azure" => Ok(CloudProvider::Azure),
            // Add other mappings as needed
            _ => Err(format!("Invalid CloudProvider: {}", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "relay_instance_type", rename_all = "lowercase")]
pub enum InstanceType {
    AwsT2Micro,
    AwsT2Nano,
    AwsT2Small,
    AwsT2Medium,
    AwsT2Large,
    GcpN1Standard1,
    GcpN1Standard2,
    GcpN1Standard4,
    AzureB1S,
    AzureB1MS,
    AzureB2S,
    AzureB2MS,
}

impl InstanceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstanceType::AwsT2Micro => "awst2micro",
            InstanceType::AwsT2Nano => "awst2nano",
            InstanceType::AwsT2Small => "awst2small",
            InstanceType::AwsT2Medium => "awst2medium",
            InstanceType::AwsT2Large => "awst2large",
            InstanceType::GcpN1Standard1 => "gcpn1standard1",
            InstanceType::GcpN1Standard2 => "gcpn1standard2",
            InstanceType::GcpN1Standard4 => "gcpn1standard4",
            InstanceType::AzureB1S => "azureb1s",
            InstanceType::AzureB1MS => "azureb1ms",
            InstanceType::AzureB2S => "azureb2s",
            InstanceType::AzureB2MS => "azureb2ms",
        }
    }

    pub fn provider_key(&self) -> String {
        match self {
            InstanceType::AwsT2Micro => "t2.micro".to_string(),
            InstanceType::AwsT2Nano => "t2.nano".to_string(),
            InstanceType::AwsT2Small => "t2.small".to_string(),
            InstanceType::AwsT2Medium => "t2.medium".to_string(),
            InstanceType::AwsT2Large => "t2.large".to_string(),
            InstanceType::GcpN1Standard1 => "n1-standard-1".to_string(),
            InstanceType::GcpN1Standard2 => "n1-standard-2".to_string(),
            InstanceType::GcpN1Standard4 => "n1-standard-4".to_string(),
            InstanceType::AzureB1S => "b1s".to_string(),
            InstanceType::AzureB1MS => "b1ms".to_string(),
            InstanceType::AzureB2S => "b2s".to_string(),
            InstanceType::AzureB2MS => "b2ms".to_string(),
        }
    }
}
