//!
//! A crate to schedule and manage containers. 
//! 
//! This crate might probably espacially interesting for testing.
//! 
//! # Basic usage
//! 
//! ```
//! use contain_rs::{
//!     client::{podman::Podman, Client, Handle},
//!     container::{Container, Image},
//! };
//! 
//! let podman = Podman::new();
//! 
//! let container = Container::from_image(Image::from_name("docker.io/library/nginx"));
//! 
//! let handle = podman.create(container);
//! 
//! handle.run()
//! 
//! // when the handle gets out of scope the container is stopped and removed
//! ```
//! 
//! # Clients
//! 
//! There are going to be different clients. Docker and podman are both planned for now.
//! 
//! Focus lies on the podman client though, which you can find [`here`](client::Podman).
//! 

pub mod container;
pub mod client;
pub mod error;
pub mod rt;
