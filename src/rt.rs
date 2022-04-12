use serde::Deserialize;

pub struct ContainerInstance {
    pub id: String,
}

impl ContainerInstance {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

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