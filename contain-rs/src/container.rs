//!
//! The container module contains all the basic data types that make up a container.
//!
//! Containers can then be run through [`clients`](crate::client::Client).
//!
//! See [Container] for further information on containers.
//!

use std::{fmt::Display, time::Duration};

use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

use crate::error::Result;

pub trait TryIntoContainer {
    fn try_into_container(self) -> Result<Container>;
}

pub trait IntoContainer {
    fn into_container(self) -> Container;
}

impl<T: TryIntoContainer> IntoContainer for T {
    fn into_container(self) -> Container {
        self.try_into_container().unwrap()
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
            command: command.to_string(),
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
#[derive(Clone)]
pub enum WaitStrategy {
    ///
    /// Waits for a log message to appear.
    ///
    LogMessage { pattern: Regex },
    ///
    /// Waits for the container to be healty.
    ///
    HealthCheck,
}

#[derive(Clone)]
pub struct Network {
    // TODO
}

#[derive(Clone, PartialEq, Eq)]
pub struct Port {
    pub number: String,
}

impl From<&str> for Port {
    fn from(s: &str) -> Self {
        Self {
            number: s.to_string(),
        }
    }
}

impl From<u32> for Port {
    fn from(s: u32) -> Self {
        Self {
            number: s.to_string(),
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

impl<T, T2> Into<EnvVar> for (T, T2)
where
    T: Into<String>,
    T2: Into<String>,
{
    fn into(self) -> EnvVar {
        EnvVar::new(self.0.into(), self.1.into())
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
    pub fn from_name(name: &str) -> Self {
        Self::from_name_and_tag(name, "latest")
    }

    pub fn from_name_and_tag(name: &str, tag: &str) -> Self {
        Image {
            name: name.to_string(),
            tag: tag.to_string(),
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

///
/// A container makes up the schedulable unit of this crate.
///
/// You can define an [Image] for it to be used and define port mappings and environment variables on it for example.
///
#[derive(Clone)]
pub struct Container {
    pub name: String,
    pub image: Image,
    pub network: Option<Network>,
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
            network: None,
            port_mappings: Vec::new(),
            env_vars: Vec::new(),
            health_check: None,
            wait_strategy: None,
            additional_wait_period: Duration::from_secs(0),
        }
    }

    ///
    /// Define a specific name for the container.
    ///
    /// In case no explicit name is defined contain-rs will generate one as the name is being used by the [Client] for interaction.
    ///
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    ///
    /// Map a port from `source` on the host to `target` in the container.
    ///
    pub fn map_port(mut self, source: impl Into<Port>, target: impl Into<Port>) -> Self {
        self.port_mappings.push(PortMapping {
            source: source.into(),
            target: target.into(),
        });
        self
    }

    ///
    /// Define an environment variable for the container.
    ///
    pub fn env_var(mut self, var: impl Into<EnvVar>) -> Self {
        let env_var = var.into();
        self.env_vars.push(env_var);
        self
    }

    ///
    /// Add a [WaitStrategy] to be used when running the container.
    ///
    pub fn wait_for(mut self, strategy: WaitStrategy) -> Self {
        self.wait_strategy = Some(strategy);
        self
    }

    ///
    /// Add some additional wait time for concidering the container healthy.
    ///
    /// Contain-rs waits this additional time after the [WaitStrategy] has been concidered successful.
    ///
    pub fn additional_wait_period(mut self, period: Duration) -> Self {
        self.additional_wait_period = period;
        self
    }

    ///
    /// Add an arbitrary healthcheck to the container.
    ///
    /// Some images may define healthchecks already, yet you can use this one to define one yourself explicitly.
    ///
    pub fn health_check(mut self, health_check: HealthCheck) -> Self {
        self.health_check = Some(health_check);
        self
    }
}

impl From<Image> for Container {
    fn from(image: Image) -> Self {
        Container::from_image(image)
    }
}
