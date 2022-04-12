use std::io::BufRead;

use crate::{container::{Container, Port}, error::Result, rt::ContainerInstance};

pub mod docker;
pub mod podman;
pub mod shared;

pub trait Client {
    type ContainerHandle: ContainerHandle;

    fn create(&self, container: Container) -> Self::ContainerHandle;
}

pub trait ContainerHandle {
    fn run(& mut self) -> Result<()>;
    fn stop(& mut self) -> Result<()>;
    fn rm(& mut self) -> Result<()>;
    fn log(& mut self) -> Result<Box<dyn BufRead>>;
    fn container(&self) -> &Container;
    fn instance(&self) -> Option<&ContainerInstance>;
    fn is_running(&self) -> bool;
}

pub trait LogStream {
    fn stream(& mut self) -> Result<Box<dyn BufRead>>;
}

