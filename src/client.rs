use std::io::BufRead;

use crate::{container::{Container, ContainerInstance}, error::Result};

pub mod docker;
pub mod podman;
pub mod shared;

pub trait Client {
    type ContainerHandle: ContainerHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle>;
}

pub trait ContainerHandle {
    fn run(& mut self) -> Result<()>;
    fn stop(& mut self) -> Result<()>;
    fn rm(& mut self) -> Result<()>;
    fn log(&self) -> Result<Box<dyn BufRead>>;
    fn container(&self) -> &Container;
    fn instance(&self) -> Option<&ContainerInstance>;
}

pub trait LogStream {
    fn stream(& mut self) -> Result<Box<dyn BufRead>>;
}

