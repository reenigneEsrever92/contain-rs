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
//! handle.run();
//! handle.wait();
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
//! # Images
//! 
//! Strictly speaking the types of this crate wouldn't exactly define an image as a thing that can be run.
//! 
//! Rather it would schedule containers. Which you can find a description of containers [`here`](container).
//! 

pub mod container;
pub mod client;
pub mod error;
pub mod rt;
