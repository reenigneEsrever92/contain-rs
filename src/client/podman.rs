use std::{io::BufRead, process::Command};

use serde::Deserialize;

use crate::{
    container::*,
    error::{Context, ErrorType, Result},
    rt::{ContainerInfo, ContainerInstance},
};

use super::{
    shared::{
        build_inspect_command, build_log_command, build_ps_command, build_rm_command,
        build_run_command, build_stop_command, do_log, run_and_wait_for_command,
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

    pub fn inspect(&self, instance: &ContainerInstance) -> Result<ContainerInfo> {
        let mut command = self.build_command();

        build_inspect_command(&mut command, instance);

        let json = run_and_wait_for_command(command)?;

        let container_info: serde_json::Result<Vec<ContainerInfo>> = serde_json::from_str(&json);

        match container_info {
            Ok(infos) => match infos.get(0) {
                Some(info) => Ok(info.to_owned()),
                None => Err(Context::new()
                    .info("message", "could not inspect container")
                    .info("json", &json)
                    .into_error(ErrorType::InspectError)),
            },
            Err(e) => Err(Context::new()
                .source(e)
                .info("message", "could not parse inspect output")
                .info("json", &json)
                .into_error(ErrorType::InspectError)),
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
/// use contain_rs::{ client::{ Client, ContainerHandle, podman::Podman }, image::postgres::Postgres };
///
/// let podman = Podman::new();
/// let container = Postgres::default().with_password("password").container();
///
/// let mut handle = podman.create(container);
///
/// assert!(handle.run().is_ok());
/// ```
///
///
impl Client for Podman {
    type ContainerHandle = PodmanHandle;

    fn create(&self, container: Container) -> Self::ContainerHandle {
        PodmanHandle {
            instance: None,
            container,
            podman: self.clone(),
        }
    }
}

pub struct PodmanHandle {
    instance: Option<ContainerInstance>,
    container: Container,
    podman: Podman,
}

impl PodmanHandle {
    pub fn try_if_running<T: FnOnce(&mut PodmanHandle, ContainerInfo) -> Result<()>>(
        &mut self,
        func: T,
    ) -> Result<()> {
        match self.instance() {
            Some(instance) => {
                let info = self.podman.inspect(instance)?;
                if info.state.running {
                    return func(self, info);
                } else {
                    Ok(())
                }
            }
            None => Ok(()),
        }
    }

    pub fn do_if_running<R, T: FnOnce(&mut PodmanHandle, ContainerInfo) -> Result<R>>(
        &mut self,
        func: T,
    ) -> Result<R> {
        match self.instance() {
            Some(instance) => {
                let info = self.podman.inspect(instance)?;
                if info.state.running {
                    return func(self, info);
                } else {
                    Err(Context::new()
                        .info("message", "Container is not running")
                        .into_error(ErrorType::ContainerStateError))
                }
            }
            None => Err(Context::new()
                .info(
                    "message",
                    "Container does not exist. Maybe you need to run it first?",
                )
                .into_error(ErrorType::ContainerStateError)),
        }
    }

    pub fn do_if_not_running<R, T: FnOnce(&mut PodmanHandle) -> Result<R>>(
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

impl ContainerHandle for PodmanHandle {
    fn run(&mut self) -> Result<()> {
        self.do_if_not_running(|handle| {
            let mut command = handle.podman.build_command();

            build_run_command(&mut command, handle.container());
            let id = run_and_wait_for_command(command)?.trim().to_string();

            handle.instance = Some(ContainerInstance::new(id));

            Ok(())
        })
    }

    fn stop(&mut self) -> Result<()> {
        self.try_if_running(|handle, info| {
            let mut command = handle.podman.build_command();

            build_stop_command(&mut command, &info.id);
            run_and_wait_for_command(command)?;

            Ok(())
        })
    }

    fn rm(&mut self) -> Result<()> {
        self.try_if_running(|handle, info| {
            let mut command = handle.podman.build_command();

            build_rm_command(&mut command, &info.id);
            run_and_wait_for_command(command)?;

            Ok(())
        })
    }

    fn log(&mut self) -> Result<Box<dyn BufRead>> {
        self.do_if_running(|handle, info| {
            let mut command = handle.podman.build_command();

            build_log_command(&mut command, &info.id);

            do_log(command)
        })
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

impl Drop for PodmanHandle {
    fn drop(&mut self) {
        self.stop().unwrap();
        self.rm().unwrap();
    }
}
