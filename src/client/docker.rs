use std::{io::BufRead, process::Command};

use crate::{
    container::{Container, ContainerInstance, WaitStrategy},
    error::{Context, ErrorType, Result},
};

use super::{
    shared::{build_run_command, run_and_wait_for_command, build_stop_command, build_rm_command},
    Client, ContainerHandle,
};

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
}

impl Client for Docker {
    type ContainerHandle = DockerHandle;

    fn create(&self, container: Container) -> Self::ContainerHandle {
        DockerHandle {
            instance: None,
            container,
            docker: self.clone(),
        }
    }
}

pub struct DockerHandle {
    instance: Option<ContainerInstance>,
    container: Container,
    docker: Docker,
}

impl DockerHandle {
    fn do_if_running<R, T: FnOnce(& mut DockerHandle) -> Result<R>>(
        & mut self,
        func: T,
    ) -> Result<R> {
        match self.instance() {
            Some(instance) => func(self),
            None => Err(Context::new()
                .info("message", "Container is not running")
                .into_error(ErrorType::ContainerStateError)),
        }
    }

    fn do_if_not_running<R, T: FnOnce(&mut DockerHandle) -> Result<R>>(
        &mut self,
        func: T,
    ) -> Result<R> {
        match self.instance() {
            Some(instance) => Err(Context::new()
                .info("message", "Container is already running")
                .info("container", &instance.id)
                .into_error(ErrorType::ContainerStateError)),
            None => func(self),
        }
    }
}

impl ContainerHandle for DockerHandle {
    fn run(&mut self) -> Result<()> {
        self.do_if_not_running(|handle| {
            let mut command = handle.docker.build_command();

            build_run_command(&mut command, handle.container());

            let id = run_and_wait_for_command(command)?;

            handle.instance = Some(ContainerInstance::new(id.trim().to_string()));

            Ok(())
        })
    }

    fn stop(&mut self) -> Result<()> {
        self.do_if_running(|handle| {
            let mut command = handle.docker.build_command();

            build_stop_command(& mut command, handle.instance().unwrap());
            run_and_wait_for_command(command)?;

            handle.instance = None;

            Ok(())
        })
    }

    fn rm(&mut self) -> Result<()> {
        self.do_if_running(|handle| {
            let mut command = handle.docker.build_command();

            build_rm_command(& mut command, handle.instance().unwrap());
            run_and_wait_for_command(command)?;

            Ok(())
        })
    }

    fn log(& mut self) -> Result<Box<dyn std::io::BufRead>> {
        todo!()
    }

    fn container(&self) -> &Container {
        &self.container
    }

    fn instance(&self) -> Option<&ContainerInstance> {
        self.instance.as_ref()
    }

    fn is_running(&self) -> bool {
        self.instance().is_some()
    }
}

impl Drop for DockerHandle {
    fn drop(&mut self) {
        if self.is_running() {
            self.rm().unwrap();
        }
    }
}