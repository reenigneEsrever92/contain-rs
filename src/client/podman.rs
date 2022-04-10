use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use log::debug;
use regex::Regex;
use serde::Deserialize;
use tracing_error::SpanTrace;

use crate::{
    container::*,
    error::{Context, ErrorType, Result},
};

use super::{
    shared::{
        build_log_command, build_ps_command, build_rm_command,
        build_run_command, build_stop_command, do_log, run_and_wait_for_command,
        wait_for, wait_for_log,
    },
    Client, ContainerHandle,
};

#[derive(Deserialize)]
pub struct PodmanContainer {
    names: Vec<String>,
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

        let result = run_and_wait_for_command(command);

        match result {
            Ok(output) => match serde_json::from_str(&output) {
                Ok(vec) => Ok(vec),
                Err(e) => Err(Context::new()
                    .source(e)
                    .info("reason", "could not parse json")
                    .info("json", &output)
                    .into_error(ErrorType::PsError)),
            },
            Err(e) => Err(Context::new().source(e).into_error(ErrorType::PsError)),
        }
    }

    fn build_command(&self) -> Command {
        Command::new(Self::BINARY)
    }

    fn build_health_check_command(&self, instance: &ContainerInstance) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("healthcheck").arg("run").arg(&instance.id);

        command
    }
}
///
/// A client implementation for podman.
///
/// ```
/// use contain_rs::{ client::{ Client, ContainerHandle, podman::{ Podman, PodmanHandle } }, image::postgres::Postgres };
///
/// let podman = Podman::new();
/// let container = Postgres::default().with_password("password").container();
///
/// let mut handle = podman.create(container).unwrap();
/// 
/// assert!(handle.run().is_ok());
/// ```
///
///
impl Client for Podman {
    type ContainerHandle = PodmanHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle> {
        let handle = PodmanHandle {
            instance: None,
            container,
            podman: self.clone(),
        };

        Ok(handle)
    }
}

pub struct PodmanHandle {
    instance: Option<ContainerInstance>,
    container: Container,
    podman: Podman,
}

impl PodmanHandle {
    pub fn do_if_running<R, T: FnOnce(&PodmanHandle, &ContainerInstance) -> Result<R>>(
        &self,
        func: T,
    ) -> Result<R> {
        match self.instance() {
            Some(instance) => func(self, instance),
            None => Err(Context::new()
                .info("message", "Container is not running")
                .into_error(ErrorType::ContainerStateError)),
        }
    }

    pub fn do_if_not_running<R, T: FnOnce(& mut PodmanHandle) -> Result<R>>(& mut self, func: T) -> Result<R> {
        match self.instance() {
            Some(instance) => Err(Context::new()
                .info("message", "Container is already running")
                .into_error(ErrorType::ContainerStateError)),
            None => func(self),
        }
    }
}

impl ContainerHandle for PodmanHandle {
    fn stop(&mut self) -> Result<()> {
        self.do_if_running(|handle, instance| {
            let mut command = handle.podman.build_command();

            build_stop_command(&mut command, &instance);
            run_and_wait_for_command(command)?;

            Ok(())
        })
    }

    fn log(&self) -> Result<Box<dyn BufRead>> {
        self.do_if_running(|handle, instance| {
            let mut command = handle.podman.build_command();

            build_log_command(&mut command, instance);

            do_log(command)
        })
    }

    fn container(&self) -> &Container {
        &self.container
    }

    fn run(&mut self) -> Result<()> {
        self.do_if_not_running(|handle| {
            let mut command = handle.podman.build_command();

            build_run_command(&mut command, handle.container());
            let id = run_and_wait_for_command(command)?.trim().to_string();

            handle.instance = Some(ContainerInstance::new(id));

            Ok(())
        })
    }

    fn rm(&mut self) -> Result<()> {
        self.do_if_running(|handle, instance| {
            let mut command = self.podman.build_command();

            build_rm_command(&mut command, instance);
            run_and_wait_for_command(command)?;

            Ok(())
        })
    }

    fn instance(&self) -> Option<&ContainerInstance> {
        self.instance.as_ref()
    }
}

impl Drop for PodmanHandle {
    fn drop(&mut self) {
        self.stop().unwrap();
        self.rm().unwrap();
    }
}
