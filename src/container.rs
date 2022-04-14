use std::{time::Duration, fmt::Display};

use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

pub mod postgres;

#[derive(Clone)]
pub struct HealthCheck {
    pub command: String,
    pub retries: Option<i32>,
    pub interval: Option<Duration>,
    pub start_period: Option<Duration>,
    pub timeout: Option<Duration>,
}

impl HealthCheck {
    pub fn new(command: String) -> Self {
        Self {
            command,
            retries: None,
            interval: None,
            start_period: None,
            timeout: None,
        }
    }

    pub fn retries(mut self, retries: i32) -> Self {
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

#[derive(Clone)]
pub enum WaitStrategy {
    LogMessage { pattern: Regex },
    HealthCheck { check: HealthCheck },
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

impl From<i32> for Port {
    fn from(s: i32) -> Self {
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

#[derive(Clone)]
pub struct Container {
    pub name: String,
    pub image: Image,
    pub network: Option<Network>,
    pub port_mappings: Vec<PortMapping>,
    pub env_vars: Vec<EnvVar>,
    pub wait_strategy: Option<WaitStrategy>,
}

impl Container {
    fn gen_hash() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect()
    }

    pub fn from_image(image: Image) -> Self {
        Container {
            name: format!("contain-rs-{}", Self::gen_hash()),
            image,
            network: None,
            port_mappings: Vec::new(),
            env_vars: Vec::new(),
            wait_strategy: None,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    pub fn map_port<'a>(
        &'a mut self,
        source: impl Into<Port>,
        target: impl Into<Port>,
    ) -> &'a Self {
        self.port_mappings.push(PortMapping {
            source: source.into(),
            target: target.into(),
        });
        self
    }

    pub fn env_var(mut self, var: impl Into<EnvVar>) -> Self {
        let env_var = var.into();
        self.env_vars.push(env_var);
        self
    }

    pub fn wait_for(mut self, strategy: WaitStrategy) -> Self {
        self.wait_strategy = Some(strategy);
        self
    }
}

impl From<Image> for Container {
    fn from(image: Image) -> Self {
        Container::from_image(image)
    }
}
