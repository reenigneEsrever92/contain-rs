use std::{process::Command, io::BufRead};

use crate::{container::{ContainerInstance, Container, WaitStrategy}, error::Result};

use super::{ContainerHandle, Client, shared::{build_run_command_from_container, run_and_wait_for_command}};

#[derive(Debug, Clone)]
pub struct Docker {
    host: Option<String>,
}

impl Docker {
    const BINARY: &'static str = "docker";

    pub fn new() -> Self {
        Self { host: None }
    }

    fn build_command(&self) -> Command {
        Command::new(Self::BINARY)
    }

    fn do_log(&self) -> Result<Box<dyn BufRead>> {
        todo!()
    }

    fn wait_for(&self, handle: &DockerHandle, strategy: &WaitStrategy) -> Result<()> {
        todo!()
    }
}

impl Client for Docker {
    type ContainerHandle = DockerHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle> {
        let mut command = self.build_command();

        build_run_command_from_container(& mut command, &container);

        let id = run_and_wait_for_command(command)?.trim().to_string();

        let handle = DockerHandle {
            instance: ContainerInstance::new(id, container),
            docker: self.clone(),
        };

        match &handle.instance.container.wait_strategy {
            Some(strategy) => self.wait_for(&handle, strategy)?,
            None => {}
        };

        Ok(handle)
    }
}

pub struct DockerHandle {
    instance: ContainerInstance,
    docker: Docker,
}

impl ContainerHandle for DockerHandle {
    fn start(& mut self) -> Result<()> {
        todo!()
    }

    fn stop(& mut self) -> Result<()> {
        todo!()
    }

    fn rm(& mut self) -> Result<()> {
        todo!()
    }

    fn log(&self) -> Result<Box<dyn std::io::BufRead>> {
        todo!()
    }

    fn container(&self) -> &Container {
        todo!()
    }
}