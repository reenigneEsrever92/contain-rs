use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerInfo {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "State")]
    pub state: ContainerState
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContainerState {
    #[serde(alias = "Running")]
    pub running: bool,
}