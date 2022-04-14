use std::{io::BufRead, process::Command};

use serde::Deserialize;

use crate::{
    container::*,
    error::{Context, ErrorType, Result},
    rt::ContainerInfo,
};

use super::{
    shared::{
        build_inspect_command, build_log_command, build_ps_command, build_rm_command,
        build_run_command, build_stop_command, do_log, run_and_wait_for_command,
        run_and_wait_for_command_infallible,
    },
    Client, ContainerHandle,
};

#[derive(Deserialize)]
pub struct PodmanContainer {
    names: Vec<String>,
}

#[derive(Deserialize)]
pub struct PodmanContainerDescription {
    name: String,
}

#[derive(Clone)]
pub struct Podman {
    host: Option<String>,
}

impl Podman {
    const BINARY: &'static str = "podman";

    pub fn new() -> Self {
        Self { host: None }
    }

    pub fn ps(&self) -> Result<Vec<PodmanContainer>> {
        let mut command = self.build_command();

        build_ps_command(&mut command);

        let output = run_and_wait_for_command_infallible(&mut command)?;

        match serde_json::from_str(&output) {
            Ok(vec) => Ok(vec),
            Err(e) => Err(Context::new()
                .source(e)
                .info("reason", "could not parse json")
                .info("json", &output)
                .into_error(ErrorType::JsonError)),
        }
    }

    pub fn exists(&self, container: &Container) -> Result<bool> {
        let mut command = self.build_command();

        command.arg("container").arg("exists").arg(&container.name);

        let output = run_and_wait_for_command(&mut command)?;

        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            Some(code) => Err(Context::new()
                .info("message", "exists command failed")
                .info("code", &code)
                .info("stderr", &String::from_utf8(output.stderr).unwrap())
                .into_error(ErrorType::CommandError)),
            None => panic!(
                "{}",
                Context::new()
                    .info("message", "command exitted with no status code")
                    .into_error(ErrorType::Unrecoverable)
            ),
        }
    }

    pub fn runs(&self, container: &Container) -> Result<bool> {
        Ok(self.inspect(container)?.is_some())
    }

    pub fn run(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        build_run_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    pub fn rm(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        build_rm_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    pub fn stop(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        build_stop_command(&mut command, container);
        run_and_wait_for_command_infallible(&mut command)?;

        Ok(())
    }

    pub fn log(&self, container: &Container) -> Result<Box<dyn BufRead>> {
        let mut command = self.build_command();

        build_log_command(&mut command, container);

        do_log(command)
    }

    pub fn inspect(&self, container: &Container) -> Result<Option<ContainerInfo>> {
        let mut command = self.build_command();

        build_inspect_command(&mut command, container);

        let json = run_and_wait_for_command_infallible(&mut command)?;

        let container_infos: Vec<ContainerInfo> = serde_json::from_str(&json).map_err(|e| {
            Context::new()
                .source(e)
                .info("message", "could not parse inspect output")
                .info("json", &json)
                .into_error(ErrorType::JsonError)
        })?;

        match container_infos.get(0) {
            Some(info) => Ok(Some(info.to_owned())),
            None => Ok(None),
        }
    }

    fn build_command(&self) -> Command {
        Command::new(Self::BINARY)
    }
}

///
/// A client implementation for podman.
///
/// ```
/// use contain_rs::{ client::{ Client, ContainerHandle, podman::Podman }, image::postgres::Postgres };
///
/// let podman = Podman::new();
/// let container = Postgres::default().with_password("password").container();
///
/// let mut handle = podman.create(container);
///
/// handle.run()
/// ```
///
///
impl Client for Podman {
    type ContainerHandle = PodmanHandle;

    fn create(&self, container: Container) -> Self::ContainerHandle {
        PodmanHandle {
            container,
            podman: self.clone(),
        }
    }
}

pub struct PodmanHandle {
    container: Container,
    podman: Podman,
}

impl ContainerHandle for PodmanHandle {
    fn run(&mut self) {
        if !self.is_running() {
            self.podman.run(&self.container).unwrap()
        }
    }

    fn stop(&mut self) {
        if self.is_running() {
            self.podman.stop(&self.container).unwrap()
        }
    }

    fn rm(&mut self) {
        if self.exists() {
            self.podman.rm(&self.container).unwrap()
        }
    }

    fn log(&mut self) -> Option<Box<dyn BufRead>> {
        if self.is_running() {
            Some(self.podman.log(&self.container).unwrap())
        } else {
            None
        }
    }

    fn container(&self) -> &Container {
        &self.container
    }

    fn is_running(&self) -> bool {
        self.exists() && self.podman.runs(&self.container).unwrap()
    }

    fn exists(&self) -> bool {
        self.podman.exists(&self.container).unwrap()
    }
}

impl Drop for PodmanHandle {
    fn drop(&mut self) {
        self.stop();
        self.rm();
    }
}
