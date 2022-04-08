use std::{io::{BufRead, Read, BufReader}, process::{Command, Child, Stdio}};

use crate::{container::Container, error::Error};

pub mod docker;
pub mod podman;
pub mod shared;

pub trait Client {
    type ContainerHandle: ContainerHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle, Error>;
}

pub trait ContainerHandle {
    type LogType: LogStream;

    fn start(& mut self) -> Result<(), Error>;
    fn stop(& mut self) -> Result<(), Error>;
    fn rm(& mut self) -> Result<(), Error>;
    fn log(&self) -> Self::LogType;
    fn container(&self) -> &Container;
}

pub trait LogStream {
    fn stream(& mut self) -> Result<Box<dyn BufRead>, Error>;
}

