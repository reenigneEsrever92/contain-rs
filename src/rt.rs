use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerInfo {
    #[serde(alias = "Names")]
    pub names: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DetailedContainerInfo {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "State")]
    pub state: ContainerState,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerState {
    #[serde(alias = "Running")]
    pub running: bool,
    #[serde(alias = "Healthcheck")]
    pub health: Option<HealthCheck>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheck {
    #[serde(alias = "Status")]
    pub status: ContainerStatus,
}

///
/// Health status of a container.
/// 
/// ```
/// use contain_rs::rt::ContainerStatus;
/// 
/// assert_eq!(serde_json::from_str::<ContainerStatus>("\"starting\"").unwrap(), ContainerStatus::Starting);
/// assert_eq!(serde_json::from_str::<ContainerStatus>("\"\"").unwrap(), ContainerStatus::Empty);
/// 
/// ```
/// 
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum ContainerStatus {
    #[serde(alias = "")]
    Empty,
    #[serde(alias = "starting")]
    Starting,
    #[serde(alias = "exited")]
    Exited,
    #[serde(alias = "healthy")]
    Healthy,
    #[serde(alias = "unhealthy")]
    Unhealthy,
}
