use std::{io, process::Output};

use crate::{container::WaitStrategy, rt::ContainerStatus};

pub type ContainerResult<T> = std::result::Result<T, ContainersError>;

#[derive(Debug, thiserror::Error)]
pub enum ContainersError {
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("Command exited with non zero exit-code")]
    CommandError(Output),
    #[error("Error parsing json")]
    JsonError(#[from] serde_json::Error),
    #[error("Unexpected container stauts: {status:?}")]
    ContainerStatusError { status: ContainerStatus },
    #[error("Container does not exist: {container_name}")]
    ContainerNotExists { container_name: String },
    #[error("Waiting for container to be ready failed. Container name: {container_name}, wait strategy: {wait_strategy:?}")]
    ContainerWaitFailed {
        container_name: String,
        wait_strategy: WaitStrategy,
    },
    #[error("Invalid image name: {name}")]
    InvalidImageName { name: String },
}
