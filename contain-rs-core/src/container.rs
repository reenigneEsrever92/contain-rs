//!
//! The container module contains all the basic data types that make up a container.
//!
//! Containers can then be run through [`clients`](crate::client::Client).
//!
//! See [Container] for further information on containers.
//!

use std::{fmt::Display, str::FromStr, time::Duration};

use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

use crate::error::{ContainerResult, ContainersError};

lazy_static! {
    static ref IMAGE_REGEX: Regex = Regex::new("([0-9a-zA-Z./]+)(:([0-9a-zA-Z.]+))?").unwrap();
}

pub trait TryIntoContainer {
    fn try_into_container(self) -> ContainerResult<Container>;
}

pub trait IntoContainer {
    fn into_container(self) -> Container;
}

impl<T: TryIntoContainer> IntoContainer for T {
    fn into_container(self) -> Container {
        self.try_into_container().unwrap()
    }
}

impl IntoContainer for Container {
    fn into_container(self) -> Container {
        self
    }
}

#[derive(Clone)]
pub struct HealthCheck {
    pub command: String,
    pub retries: Option<u32>,
    pub interval: Option<Duration>,
    pub start_period: Option<Duration>,
    pub timeout: Option<Duration>,
}

impl HealthCheck {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.into(),
            retries: None,
            interval: None,
            start_period: None,
            timeout: None,
        }
    }

    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn start_period(mut self, start_period: Duration) -> Self {
        self.start_period = Some(start_period);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

///
/// A wait strategy can be used to wait for a cotnainer to be ready.
///
#[derive(Clone, Debug)]
pub enum WaitStrategy {
    ///
    /// Waits for a log message to appear.
    ///
    LogMessage { pattern: Regex },
    ///
    /// Waits for the container to be healty.
    ///
    HealthCheck,
    ///
    /// Wait for some amount of time.
    ///
    WaitTime { duration: Duration },
}

#[derive(Clone)]
pub struct Network {
    // TODO
}

#[derive(Clone, PartialEq, Eq)]
pub struct Port {
    pub number: String,
}

impl<T> From<T> for Port
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self {
            number: value.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct PortMapping {
    pub source: Port,
    pub target: Port,
}

#[derive(Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl EnvVar {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

impl<K, V> From<(K, V)> for EnvVar
where
    K: Into<String>,
    V: Into<String>,
{
    fn from(value: (K, V)) -> Self {
        EnvVar::new(value.0.into(), value.1.into())
    }
}

#[derive(Clone)]
pub struct Image {
    pub name: String,
    pub tag: String,
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.tag)
    }
}

impl Image {
    pub fn from_name_and_tag(name: &str, tag: &str) -> Self {
        Image {
            name: name.to_string(),
            tag: tag.to_string(),
        }
    }
}

impl FromStr for Image {
    type Err = ContainersError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = IMAGE_REGEX.captures(s);

        if let Some(cap) = caps {
            Ok(Self::from_name_and_tag(
                cap.get(1).unwrap().as_str(),
                cap.get(3).map(|m| m.as_str()).unwrap_or("latest"),
            ))
        } else {
            Err(ContainersError::InvalidImageName {
                name: s.to_string(),
            })
        }
    }
}

impl From<Image> for String {
    fn from(i: Image) -> Self {
        format!("{}:{}", i.name, i.tag)
    }
}

impl From<&Image> for String {
    fn from(i: &Image) -> Self {
        format!("{}:{}", i.name, i.tag)
    }
}

#[derive(Clone)]
pub enum Volume {
    Mount {
        host_path: String,
        mount_point: String,
    },
    Named {
        name: String,
        mount_point: String,
    },
}

///
/// A container makes up the schedulable unit of this crate.
///
/// You can define an [Image] for it to be used and define port mappings and environment variables on it for example.
///
#[derive(Clone)]
pub struct Container {
    pub name: String,
    pub image: Image,
    pub command: Vec<String>,
    pub network: Option<Network>,
    pub volumes: Vec<Volume>,
    pub port_mappings: Vec<PortMapping>,
    pub env_vars: Vec<EnvVar>,
    pub health_check: Option<HealthCheck>,
    pub wait_strategy: Option<WaitStrategy>,
    pub additional_wait_period: Duration,
}

impl Container {
    fn gen_hash() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect()
    }

    ///
    /// Creates a new container from and [Image]
    ///
    pub fn from_image(image: Image) -> Self {
        Container {
            name: format!("contain-rs-{}", Self::gen_hash()),
            image,
            command: Vec::new(),
            network: None,
            port_mappings: Vec::new(),
            env_vars: Vec::new(),
            volumes: Vec::new(),
            health_check: None,
            wait_strategy: None,
            additional_wait_period: Duration::from_secs(0),
        }
    }

    ///
    /// Define a specific name for the container.
    ///
    /// In case no explicit name is defined contain-rs will generate one as the name is being used by the [crate::client::Client] for interaction.
    ///
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.into();
        self
    }

    ///
    /// Define an explicit command to run in the container.
    ///
    pub fn command(&mut self, command: Vec<String>) -> &mut Self {
        self.command = command;
        self
    }

    pub fn arg<T: Into<String>>(&mut self, arg: T) -> &mut Self {
        self.command.push(arg.into());
        self
    }

    pub fn map_ports<T, T2>(&mut self, ports: &[(T, T2)]) -> &mut Self
    where
        T: Into<Port> + Clone,
        T2: Into<Port> + Clone,
    {
        self.port_mappings = ports
            .iter()
            .cloned()
            .map(|mapping| PortMapping {
                source: mapping.0.into(),
                target: mapping.1.into(),
            })
            .collect();

        self
    }

    pub fn volume(&mut self, name: &str, mount_point: &str) -> &mut Self {
        self.volumes.push(Volume::Named {
            name: name.to_string(),
            mount_point: mount_point.to_string(),
        });

        self
    }

    pub fn mount(&mut self, host_path: &str, mount_point: &str) -> &mut Self {
        self.volumes.push(Volume::Mount {
            host_path: host_path.to_string(),
            mount_point: mount_point.to_string(),
        });

        self
    }

    ///
    /// Map a port from `source` on the host to `target` in the container.
    ///
    pub fn map_port(&mut self, source: impl Into<Port>, target: impl Into<Port>) -> &mut Self {
        self.port_mappings.push(PortMapping {
            source: source.into(),
            target: target.into(),
        });
        self
    }

    ///
    /// Define an environment variable for the container.
    ///
    pub fn env_var<T: Into<String>>(&mut self, name: T, value: T) -> &mut Self {
        self.env_vars.push((name, value).into());
        self
    }

    ///
    /// Add a [WaitStrategy] to be used when running the container.
    ///
    pub fn wait_for(&mut self, strategy: WaitStrategy) -> &mut Self {
        self.wait_strategy = Some(strategy);
        self
    }

    ///
    /// Add some additional wait time for concidering the container healthy.
    ///
    /// Contain-rs waits this additional time after the [WaitStrategy] has been concidered successful.
    ///
    pub fn additional_wait_period(&mut self, period: Duration) -> &mut Self {
        self.additional_wait_period = period;
        self
    }

    ///
    /// Add an arbitrary healthcheck to the container.
    ///
    /// Some images may define healthchecks already, yet you can use this one to define one yourself explicitly.
    ///
    pub fn health_check(&mut self, health_check: HealthCheck) -> &mut Self {
        self.health_check = Some(health_check);
        self
    }
}

impl From<Image> for Container {
    fn from(image: Image) -> Self {
        Container::from_image(image)
    }
}
