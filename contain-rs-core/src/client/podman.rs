use std::process::Command;

use crate::{container::*, error::ContainerResult, rt::DetailedContainerInfo};

use super::{
    shared::{
        build_rm_command, build_run_command, build_stop_command, do_log, inspect,
        run_and_wait_for_command_infallible, wait_for,
    },
    Client, ContainerHandle, Log,
};

///
/// The Podman struct is used for acessing the podman cli.
///
/// ```
/// use contain_rs_core::{
///     client::{podman::Podman, Client, Handle},
///     container::{Container, Image, HealthCheck, WaitStrategy},
/// };
/// use std::str::FromStr;
///
/// let client = Podman::new();
///
/// let mut container = Container::from_image(Image::from_str("docker.io/library/nginx").unwrap());
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
pub struct Podman {
    host: Option<String>,
}

impl Podman {
    const BINARY: &'static str = "podman";

    pub fn new() -> Self {
        Self { host: None }
    }

    fn build_command(&self) -> Command {
        Command::new(Self::BINARY)
    }
}

impl Default for Podman {
    fn default() -> Self {
        Self::new()
    }
}

impl Client for Podman {
    type ClientType = Self;

    fn command(&self) -> Command {
        self.build_command()
    }

    fn create<C: IntoContainer>(&self, container: C) -> ContainerHandle<Podman> {
        ContainerHandle {
            client: self.to_owned(),
            container: container.into_container(),
        }
    }

    fn run(&self, container: &Container) -> ContainerResult<()> {
        let mut command = self.build_command();

        build_run_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    fn stop(&self, container: &Container) -> ContainerResult<()> {
        let mut command = self.build_command();

        build_stop_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    fn rm(&self, container: &Container) -> ContainerResult<()> {
        let mut command = self.build_command();

        build_rm_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    fn log(&self, container: &Container) -> ContainerResult<Option<Log>> {
        if self.runs(container)? {
            Ok(Some(do_log(self, container)?))
        } else {
            Ok(None)
        }
    }

    fn inspect(&self, container: &Container) -> ContainerResult<Option<DetailedContainerInfo>> {
        inspect(self, container)
    }

    fn exists(&self, container: &Container) -> ContainerResult<bool> {
        Ok(self.inspect(container)?.is_some())
    }

    fn runs(&self, container: &Container) -> ContainerResult<bool> {
        match self.inspect(container)? {
            Some(detail) => Ok(detail.state.running),
            None => Ok(false),
        }
    }

    fn wait(&self, container: &Container) -> ContainerResult<()> {
        wait_for(self, container)
    }
}
