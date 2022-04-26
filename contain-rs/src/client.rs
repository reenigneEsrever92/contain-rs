//!
//! This module contains the traits that represent the bare functiionality of a client.
//!
//! See [Podman] for usage of the podman client implementation.
//!

use std::{
    io::{BufRead, BufReader},
    process::{Child, Command},
};

use crate::{container::Container, error::Result, rt::DetailedContainerInfo};

pub mod docker;
pub mod podman;
pub mod shared;

pub use self::docker::Docker;
pub use self::podman::Podman;

///
/// The client Trait represents a way to access a client.
///
/// It's implemented by the [Podman] struct for example. And will be for docker as well.
/// If you do that for any specific Type you get [`handles`](Handle) for free.
///
///
pub trait Client: Clone {
    type ClientType: Client;

    fn command(&self) -> Command;
    fn create(&self, container: Container) -> ContainerHandle<Self::ClientType>;
    fn run(&self, container: &Container) -> Result<()>;
    fn stop(&self, container: &Container) -> Result<()>;
    fn rm(&self, container: &Container) -> Result<()>;
    fn log(&self, container: &Container) -> Result<Option<Log>>;
    fn inspect(&self, container: &Container) -> Result<Option<DetailedContainerInfo>>;
    fn exists(&self, container: &Container) -> Result<bool>;
    fn runs(&self, container: &Container) -> Result<bool>;
    fn wait(&self, container: &Container) -> Result<()>;
}

///
/// A handle is a way to interact with a container.
///
/// When you create a container using a [Client] it will return one of these.
/// The handle automatically stops and removes the container, when it goes out of scope.
///
pub trait Handle {
    fn run(&self);
    fn stop(&self);
    fn rm(&self);
    fn log(&self) -> Option<Log>;
    fn container(&self) -> &Container;
    fn is_running(&self) -> bool;
    fn exists(&self) -> bool;
}

pub struct Log {
    pub child: Child,
}

impl Drop for Log {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

impl Log {
    fn stream(&mut self) -> Option<impl BufRead> {
        self.child
            .stdout
            .take()
            .map(|stdout| BufReader::new(stdout))
    }
}

pub struct ContainerHandle<T: Client> {
    client: T,
    container: Container,
}

impl<T: Client> Handle for ContainerHandle<T> {
    fn run(&self) {
        if !self.is_running() {
            self.client.run(&self.container).unwrap();
            self.client.wait(&self.container).unwrap();
        }
    }

    fn stop(&self) {
        if self.is_running() {
            self.client.stop(&self.container).unwrap()
        }
    }

    fn rm(&self) {
        if self.exists() {
            self.client.rm(&self.container).unwrap()
        }
    }

    fn log(&self) -> Option<Log> {
        if self.is_running() {
            self.client.log(&self.container).unwrap()
        } else {
            None
        }
    }

    fn container(&self) -> &Container {
        &self.container
    }

    fn is_running(&self) -> bool {
        self.exists() && self.client.runs(&self.container).unwrap()
    }

    fn exists(&self) -> bool {
        self.client.exists(&self.container).unwrap()
    }
}

impl<T: Client> Drop for ContainerHandle<T> {
    fn drop(&mut self) {
        self.stop();
        self.rm();
    }
}
