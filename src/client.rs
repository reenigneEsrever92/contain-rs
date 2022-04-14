use std::io::BufRead;

use crate::{container::Container, error::Result};

pub mod docker;
pub mod podman;
pub mod shared;

pub trait Client {
    type ContainerHandle: ContainerHandle;

    fn create(&self, container: Container) -> Self::ContainerHandle;
}

pub trait ContainerHandle {
    fn run(& mut self);
    fn stop(& mut self);
    fn rm(& mut self);
    fn log(& mut self) -> Option<Box<dyn BufRead>>;
    fn container(&self) -> &Container;
    fn is_running(&self) -> bool;
    fn exists(&self) -> bool;
}

pub trait LogStream {
    fn stream(& mut self) -> Result<Box<dyn BufRead>>;
}

