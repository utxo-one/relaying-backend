use crate::models::cloud_instance::{CloudInstance, LaunchCloudInstance};
use rusoto_core::HttpClient;
use rusoto_credential::{InstanceMetadataProvider, ProvideAwsCredentials};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use rusoto_signature::Region;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

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

    // Create tags for the instance with a name
    let mut tags = HashMap::new();
    tags.insert("Name".to_string(), launch.name.to_string());

    // Create the request to launch an instance
    let run_instance_req = rusoto_ec2::RunInstancesRequest {
        image_id: Some(launch.image_id.to_string()),
        instance_type: Some(launch.instance_type.to_string()),
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

    // Launch the instance
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

        sleep(std::time::Duration::from_secs(3)).await;
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::util::generators::generate_random_string;

    #[tokio::test]
    async fn test_launch_instance() {
        let instance_name = generate_random_string(10).await;
        let launch = LaunchCloudInstance {
            name: instance_name.clone(),
            image_id: env!("STRFRY_AMI").to_string(),
            instance_type: "t2.nano".to_string(),
            implementation: "strfry".to_string(),
        };

        let instance = launch_instance(launch)
            .await
            .expect("Failed to launch instance");

        assert!(instance.id.starts_with("i-"));
        assert!(!instance.ip_address.is_empty());
    }
}
