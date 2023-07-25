use super::relay::RelayImplementation;
use rusoto_core::HttpClient;
use rusoto_credential::{InstanceMetadataProvider, ProvideAwsCredentials};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use rusoto_signature::Region;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::sleep;

// -----------------------------------------------------------------------------
// Models
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Functions
// -----------------------------------------------------------------------------

async fn get_credentials() -> Result<rusoto_credential::AwsCredentials, String> {
    let provider = InstanceMetadataProvider::new();
    let credentials = provider
        .credentials()
        .await
        .map_err(|err| format!("Failed to get credentials: {:?}", err))?;

    Ok(credentials)
}

pub async fn launch_instance(launch: LaunchCloudInstance) -> Result<CloudInstance, String> {
    // Create an EC2 client
    dotenvy::dotenv().ok();
    let env_provider = rusoto_credential::EnvironmentProvider::default();
    let ec2_client = Ec2Client::new_with(HttpClient::new().unwrap(), env_provider, Region::UsEast1);
    let run_instance_req = create_instance_request(launch);
    let response = ec2_client.run_instances(run_instance_req).await;

    // Check if the instance was launched successfully
    let reservation = response.map_err(|err| format!("Error launching instance: {:?}", err))?;
    if let Some(instance) = reservation.instances {
        if let Some(instance_data) = instance.first() {
            let instance_id = instance_data
                .instance_id
                .as_ref()
                .ok_or("Instance ID not found")?;

            // Wait for the instance to have an IP address
            let ip_address = wait_for_instance_ready(&ec2_client, instance_id).await?;

            return Ok(CloudInstance {
                id: instance_id.clone(),
                ip_address: ip_address.clone(),
            });
        }
    }
    Err("Failed to get the instance details.".to_string())
}

pub async fn terminate_instance(instance_id: &str) -> Result<(), String> {
    // Create an EC2 client
    dotenvy::dotenv().ok();
    let env_provider = rusoto_credential::EnvironmentProvider::default();
    let ec2_client = Ec2Client::new_with(HttpClient::new().unwrap(), env_provider, Region::UsEast1);

    // Create the request to terminate an instance
    let terminate_instance_req = rusoto_ec2::TerminateInstancesRequest {
        instance_ids: vec![instance_id.to_string()],
        ..Default::default()
    };

    // Terminate the instance
    let response = ec2_client.terminate_instances(terminate_instance_req).await;

    // Check if the instance was terminated successfully
    let result = response.map_err(|err| format!("Error terminating instance: {:?}", err))?;
    if let Some(terminating_instances) = result.terminating_instances {
        if let Some(instance) = terminating_instances.first() {
            if instance
                .instance_id
                .as_ref()
                .map_or(false, |id| id == instance_id)
            {
                return Ok(());
            }
        }
    }
    Err("Failed to terminate the instance.".to_string())
}

fn create_instance_request(launch: LaunchCloudInstance) -> rusoto_ec2::RunInstancesRequest {
    // Create tags for the instance with a name
    let mut tags = HashMap::new();
    tags.insert("Name".to_string(), launch.name.to_string());

    // Create the request to launch an instance
    let run_instance_req = rusoto_ec2::RunInstancesRequest {
        image_id: Some(launch.image_id.to_string()),
        instance_type: Some(launch.instance_type.provider_key()),
        min_count: 1,
        max_count: 1,
        tag_specifications: Some(vec![rusoto_ec2::TagSpecification {
            resource_type: Some("instance".to_string()),
            tags: Some(
                tags.iter()
                    .map(|(key, value)| rusoto_ec2::Tag {
                        key: Some(key.to_string()),
                        value: Some(value.to_string()),
                    })
                    .collect(),
            ),
        }]),
        ..Default::default()
    };

    run_instance_req
}

async fn wait_for_instance_ready(
    ec2_client: &Ec2Client,
    instance_id: &str,
) -> Result<String, String> {
    loop {
        let describe_instances_req = DescribeInstancesRequest {
            instance_ids: Some(vec![instance_id.to_string()]),
            ..Default::default()
        };

        let response = ec2_client.describe_instances(describe_instances_req).await;

        if let Ok(result) = response {
            if let Some(reservations) = result.reservations {
                if let Some(instance) = reservations[0].instances.as_ref() {
                    if instance[0]
                        .public_ip_address
                        .as_ref()
                        .map_or(false, |ip| !ip.is_empty())
                    {
                        return Ok(instance[0].public_ip_address.clone().unwrap());
                    }
                }
            }
        }

        sleep(std::time::Duration::from_secs(1)).await;
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::generate_random_string;

    #[tokio::test]
    async fn test_launch_and_terminate_instance() {
        let instance_name = generate_random_string(10).await;
        let launch = LaunchCloudInstance {
            name: instance_name.clone(),
            image_id: dotenvy::var("STRFRY_AMI").unwrap(),
            instance_type: InstanceType::AwsT2Nano,
            implementation: RelayImplementation::Strfry,
        };

        let instance = launch_instance(launch)
            .await
            .expect("Failed to launch instance");

        assert!(instance.id.starts_with("i-"));
        assert!(!instance.ip_address.is_empty());

        let terminate_result = terminate_instance(&instance.id).await;

        assert!(terminate_result.is_ok());
    }
}
