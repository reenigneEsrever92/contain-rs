use std::{io::BufRead, process::Command};

use crate::{
    container::*,
    error::{Context, ErrorType, Result},
    rt::{ContainerInfo, ProcessState},
};

use super::{
    shared::{
        build_inspect_command, build_log_command, build_ps_command, build_rm_command,
        build_run_command, build_stop_command, do_log, run_and_wait_for_command,
        run_and_wait_for_command_infallible, wait_for,
    },
    Client, ContainerHandle, Log,
};

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

///
/// A client implementation for podman.
///
/// ```
/// use contain_rs::{
///     client::{podman::Podman, Client, Handle},
///     container::{postgres::Postgres, Container, Image},
/// };
///
/// let podman = Podman::new();
/// let container = Postgres::default().with_password("password").container();
///
/// let mut handle = podman.create(container);
///
/// handle.run()
/// ```
///
impl Client for Podman {
    type ClientType = Self;

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
            let mut command = self.build_command();

            build_log_command(&mut command, container);

            Ok(Some(do_log(&mut command)?))
        } else {
            Ok(None)
        }
    }

    fn inspect(&self, container: &Container) -> Result<Option<ContainerInfo>> {
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

    fn exists(&self, container: &Container) -> Result<bool> {
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

    fn runs(&self, container: &Container) -> Result<bool> {
        Ok(self.inspect(container)?.is_some())
    }

    fn ps(&self) -> Result<Vec<ProcessState>> {
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

    fn wait(&self, container: &Container) -> Result<()> {
        let mut command = self.build_command();

        wait_for(command, container)
    }
}
