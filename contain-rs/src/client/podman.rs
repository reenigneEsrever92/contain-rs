use std::process::Command;

use crate::{container::*, error::Result, rt::DetailedContainerInfo};

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

impl Client for Podman {
    type ClientType = Self;

    fn command(&self) -> Command {
        self.build_command()
    }

    fn create(&self, container: Container) -> ContainerHandle<Podman> {
        ContainerHandle {
            client: self.to_owned(),
            container: container,
        }
    }

    fn run(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        build_run_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    fn stop(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        build_stop_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    fn rm(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        build_rm_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

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
