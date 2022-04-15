use std::{
    io::{BufRead, BufReader},
    marker::PhantomData,
    process::{Child, Command},
};

use crate::{
    container::{Container, WaitStrategy},
    error::Result,
    rt::{ContainerInfo, ProcessState},
};

pub mod docker;
pub mod podman;
pub mod shared;

pub trait Client: Clone {
    type ClientType: Client;

    fn create(&self, container: Container) -> ContainerHandle<Self::ClientType>;
    fn run(&self, container: &Container) -> Result<()>;
    fn stop(&self, container: &Container) -> Result<()>;
    fn rm(&self, container: &Container) -> Result<()>;
    fn log(&self, container: &Container) -> Result<Option<Log>>;
    fn inspect(&self, container: &Container) -> Result<Option<ContainerInfo>>;
    fn exists(&self, container: &Container) -> Result<bool>;
    fn runs(&self, container: &Container) -> Result<bool>;
    fn ps(&self) -> Result<Vec<ProcessState>>;
    fn wait(&self, container: &Container) -> Result<()>;
}

pub trait Handle {
    fn run(&mut self);
    fn stop(&mut self);
    fn rm(&mut self);
    fn log(&mut self) -> Option<Log>;
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
    fn run(&mut self) {
        if !self.is_running() {
            self.client.run(&self.container).unwrap();
            self.client.wait(&self.container).unwrap();
        }
    }

    fn stop(&mut self) {
        if self.is_running() {
            self.client.stop(&self.container).unwrap()
        }
    }

    fn rm(&mut self) {
        if self.exists() {
            self.client.rm(&self.container).unwrap()
        }
    }

    fn log(&mut self) -> Option<Log> {
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

pub trait LogStream {
    fn stream(&mut self) -> Result<Box<dyn BufRead>>;
}
