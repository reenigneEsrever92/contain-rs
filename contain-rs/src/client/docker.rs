use std::process::Command;

use crate::{
    container::{Container, IntoContainer},
    error::Result,
    rt::DetailedContainerInfo,
};

use super::{
    shared::{
        build_rm_command, build_run_command, build_stop_command, do_log, inspect,
        run_and_wait_for_command, wait_for,
    },
    Client, ContainerHandle, Log,
};

///
/// The Docker struct is used for acessing the docker cli.
///
/// ```
/// use contain_rs::{
///     client::{docker::Docker, Client, Handle},
///     container::{Container, Image, HealthCheck, WaitStrategy},
/// };
///
/// let client = Docker::new();
///
/// let mut container = Container::from_image(Image::from_name("docker.io/library/nginx"));
///     
/// container.health_check(HealthCheck::new("curl http://localhost || exit 1"))
///     .wait_for(WaitStrategy::HealthCheck);
///
/// client.run(&container).unwrap();
/// client.wait(&container).unwrap();
/// client.rm(&container).unwrap();
/// ```
///
#[allow(dead_code)]
#[derive(Clone)]
pub struct Docker {
    host: Option<String>,
}

impl Docker {
    pub fn new() -> Self {
        Self { host: None }
    }

    fn build_command(&self) -> Command {
        Command::new("docker")
    }
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}

impl Client for Docker {
    type ClientType = Docker;

    fn command(&self) -> Command {
        self.build_command()
    }

    fn create<C: IntoContainer>(&self, container: C) -> super::ContainerHandle<Self::ClientType> {
        ContainerHandle {
            client: self.clone(),
            container: container.into_container(),
        }
    }

    fn run(&self, container: &Container) -> Result<()> {
        let mut cmd = self.build_command();

        build_run_command(&mut cmd, container);
        run_and_wait_for_command(&mut cmd)?;

        Ok(())
    }

    fn stop(&self, container: &Container) -> Result<()> {
        let mut cmd = self.build_command();

        build_stop_command(&mut cmd, container);
        run_and_wait_for_command(&mut cmd)?;

        Ok(())
    }

    fn rm(&self, container: &Container) -> Result<()> {
        let mut cmd = self.build_command();

        build_rm_command(&mut cmd, container);
        run_and_wait_for_command(&mut cmd)?;

        Ok(())
    }

    fn log(&self, container: &Container) -> Result<Option<Log>> {
        if self.runs(container)? {
            Ok(Some(do_log(self, container)?))
        } else {
            Ok(None)
        }
    }

    fn inspect(&self, container: &Container) -> Result<Option<DetailedContainerInfo>> {
        inspect(self, container)
    }

    fn exists(&self, container: &Container) -> Result<bool> {
        Ok(self.inspect(container)?.is_some())
    }

    fn runs(&self, container: &Container) -> Result<bool> {
        match self.inspect(container)? {
            Some(detail) => Ok(detail.state.running),
            None => Ok(false),
        }
    }

    fn wait(&self, container: &Container) -> Result<()> {
        wait_for(self, container)
    }
}
