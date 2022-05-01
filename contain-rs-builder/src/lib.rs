use std::time::Duration;

use contain_rs::{
    container::{Container, Image, IntoContainer, TryIntoContainer},
    error::{Context, ErrorType::ContainerError, Result},
};

#[derive(Clone)]
pub enum Component {
    Image(String),
    EnvVar(String, String),
    Port(u32, u32),
    HealthCheck(
        String,
        Option<u32>,
        Option<Duration>,
        Option<Duration>,
        Option<Duration>,
    ),
    Wait(WaitStrategy),
}

#[derive(Clone)]
pub enum WaitStrategy {
    HealthCheck,
    LogMessage(String),
}

pub struct ContainerDecleration(Vec<Component>);

pub fn container<T: IntoIterator<Item = Component>>(components: T) -> ContainerDecleration {
    ContainerDecleration(components.into_iter().collect())
}

pub fn env_var(key: &str, value: &str) -> Component {
    Component::EnvVar(key.to_string(), value.to_string())
}

pub fn image(name: &str) -> Component {
    Component::Image(name.to_string())
}

pub fn port(host: u32, target: u32) -> Component {
    Component::Port(host, target)
}

pub fn health_check(
    cmd: &str,
    retries: Option<u32>,
    interval: Option<Duration>,
    start_period: Option<Duration>,
    timeout: Option<Duration>,
) -> Component {
    Component::HealthCheck(cmd.to_string(), retries, interval, start_period, timeout)
}

pub fn wait_for_log(message: &str) -> Component {
    Component::Wait(WaitStrategy::LogMessage(message.to_string()))
}

pub fn wait_for_healthcheck() -> Component {
    Component::Wait(WaitStrategy::HealthCheck)
}

impl TryIntoContainer for ContainerDecleration {
    fn try_into_container(self) -> contain_rs::error::Result<Container> {
        Ok(Container::from_image(self.get_image()?))
    }
}

impl ContainerDecleration {
    fn get_image(&self) -> Result<Image> {
        let names: Vec<String> = self
            .0
            .iter()
            .flat_map(|cmp| match cmp {
                Component::Image(name) => Some(name.clone()),
                _ => None,
            })
            .collect();

        if names.len() > 1 {
            return Err(Context::new()
                .info("message", "Mutliple image names specified")
                .into_error(ContainerError));
        }

        match names.get(0) {
            Some(name) => Ok(Image::from_name(name)),
            None => Err(Context::new()
                .info("message", "No image name specified")
                .into_error(ContainerError)),
        }
    }
}
