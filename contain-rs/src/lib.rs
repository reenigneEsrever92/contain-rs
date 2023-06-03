#![doc = include_str!("../README.md")]

pub use contain_rs_core::{
    container::{
        Container, EnvVar, HealthCheck, Image, IntoContainer, Port, PortMapping, WaitStrategy,
    },
    Regex,
};

pub use contain_rs_core::client::{
    docker::Docker, podman::Podman, Client, ContainerHandle, Handle,
};

#[cfg(feature = "macros")]
pub use contain_rs_macro::ContainerImpl;
