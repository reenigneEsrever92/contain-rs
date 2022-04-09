use std::io::BufRead;

use crate::{container::Container, error::Result};

pub mod docker;
pub mod podman;

pub trait Client {
    type ContainerHandle: ContainerHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle>;
}

pub trait ContainerHandle {
    fn start(& mut self) -> Result<()>;
    fn stop(& mut self) -> Result<()>;
    fn rm(& mut self) -> Result<()>;
    fn log(&self) -> Result<Box<dyn BufRead>>;
    fn container(&self) -> &Container;
}

pub trait LogStream {
    fn stream(& mut self) -> Result<Box<dyn BufRead>>;
}

