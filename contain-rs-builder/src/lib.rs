use std::time::Duration;

use contain_rs::container::{Container, TryIntoContainer};

#[derive(Clone)]
pub enum Component {
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

pub struct ContainerDecleration(Image, Vec<Component>);

pub struct Image(String);

pub fn declare<C>(image: Image, components: C) -> ContainerDecleration
where
    C: IntoIterator<Item = Component>,
{
    ContainerDecleration(image, components.into_iter().collect())
}

pub fn env_var(key: &str, value: &str) -> Component {
    Component::EnvVar(key.to_string(), value.to_string())
}

pub fn image(name: &str) -> Image {
    Image(name.to_string())
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
        Ok(Container::from_image(
            contain_rs::container::Image::from_name(&self.0 .0),
        ))
    }
}

impl ContainerDecleration {}
