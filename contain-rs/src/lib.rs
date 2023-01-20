//!
//! A crate to schedule containers.
//!
//! This crate might probably be espacially interesting for testing.
//!
//! # Basic usage
//!
//! ```
//! use contain_rs::{Podman, Client, Handle, Container, Image};
//! use std::str::FromStr;
//!
//! let podman = Podman::new();
//!
//! let container = Container::from_image(Image::from_str("docker.io/library/nginx").unwrap());
//!
//! let handle = podman.create(container);
//!
//! handle.run();
//! handle.wait();
//!
//! // when the handle gets out of scope the container is stopped and removed
//! ```
//!
//! # Clients
//!
//! Clients are used for scheduling containers. There are currently two implementations available.
//! One of them works with docker the other one uses podman.
//!
//! # Images
//!
//! Containers need image to run. You can create images like so:
//!
//! ```
//! use contain_rs::Image;
//! use std::str::FromStr;
//!
//! let image = Image::from_str("docker.io/library/nginx");
//!
//! assert!(image.is_ok());
//!
//! let latest = Image::from_str("docker.io/library/nginx:latest");
//!
//! assert!(latest.is_ok());
//! ```
//!

pub use contain_rs_core::{
    container::{
        Container, EnvVar, HealthCheck, Image, IntoContainer, Port, PortMapping, WaitStrategy,
    },
    Regex,
};

pub use contain_rs_core::client::{docker::Docker, podman::Podman, Client, Handle};

#[cfg(feature = "macros")]
pub use contain_rs_macro::ContainerImpl;
