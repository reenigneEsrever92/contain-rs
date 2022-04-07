use std::{collections::HashMap, ops::Deref, process::Command};

use regex::Regex;

use crate::{client::ContainerHandle, image::Image};

pub enum WaitStrategy {
    LogMessage { pattern: Regex },
}

pub struct Network {
    // TODO
}

pub struct Port {
    number: String,
}

pub struct PortMapping {
    source: Port,
    target: Port,
}

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
    T2: Into<String>
{
    fn into(self) -> EnvVar {
        EnvVar::new(self.0.into(), self.1.into())
    }
}

pub struct Container {
    pub image: Image,
    pub network: Option<Network>,
    pub port_mappings: Vec<PortMapping>,
    pub env_vars: Vec<EnvVar>,
    pub wait_strategy: Option<WaitStrategy>,
}

impl Container {
    pub fn from_image(image: Image) -> Self {
        Container {
            image,
            network: None,
            port_mappings: Vec::new(),
            env_vars: Vec::new(),
            wait_strategy: None,
        }
    }

    pub fn expose_port<'a>(
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
