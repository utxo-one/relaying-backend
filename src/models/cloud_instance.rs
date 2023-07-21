pub struct CloudInstance {
    pub id: String,
    pub ip_address: String,
}

pub struct LaunchCloudInstance {
    pub name: String,
    pub image_id: String,
    pub instance_type: String,
    pub implementation: String,
}
