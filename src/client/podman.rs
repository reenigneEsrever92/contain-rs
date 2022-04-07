use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, ExitStatus, Stdio},
};

use log::debug;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;

use crate::{container::*, error::Error};

use super::{Client, ContainerHandle, LogStream, shared::SharedLogStream};

#[derive(Clone)]
pub struct Podman {
    host: Option<String>,
}

impl Podman {
    const BINARY: &'static str = "podman";

    pub fn new() -> Self {
        Self { host: None }
    }

    fn build_run_command(&self, container: &Container) -> Command {
        let mut command = Command::new(Self::BINARY);

        self.add_run_args(&mut command);
        self.add_env_var_args(&mut command, container);
        self.add_image_arg(&mut command, container);

        command
    }

    fn add_run_args(&self, command: &mut Command) {
        command.arg("run").arg("-d");
    }

    fn add_env_var_args(&self, command: &mut Command, container: &Container) {
        container.env_vars.iter().for_each(|env_var| {
            command
                .arg("-e")
                .arg(format!("{}={}", env_var.key, env_var.value));
        });
    }

    fn add_image_arg(&self, command: &mut Command, container: &Container) {
        command.arg(String::from(&container.image));
    }

    fn run_command(&self, command: &mut Command) -> Child {
        debug!("Run command: {:?}", command);

        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    }

    fn run_and_wait_for_command(&self, command: &mut Command) -> Result<String, std::io::Error> {
        debug!("Run and wait for command: {:?}", command);

        let result = command
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .map(|r| String::from_utf8(r.stdout).unwrap());

        debug!("Command result: {:?}", result);

        result
    }

    fn wait_for(&self, handle: &PodmanHandle, strategy: &WaitStrategy) -> Result<(), Error> {
        match strategy {
            WaitStrategy::LogMessage { pattern } => self.wait_for_log(handle, &pattern),
        }
    }

    fn wait_for_log(&self, handle: &PodmanHandle, pattern: &Regex) -> Result<(), Error> {
        let stream = handle.log().stream()?;

        for line in stream.lines() {
            match line {
                Ok(s) => {
                    debug!("Searching for LogMessage pattern: {pattern}, in: {s}");
                    if pattern.is_match(&s) {
                        debug!("Found pattern in line: {s}");
                        return Ok(());
                    }
                }
                Err(e) => todo!(),
            };
        }

        Err(Error::WaitError {
            message: "Could not match pattern in container output.".to_string(),
        })
    }
}

impl Client for Podman {
    type ContainerHandle = PodmanHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle, Error> {
        let id = self
            .run_and_wait_for_command(&mut self.build_run_command(&container))?
            .trim()
            .to_string();

        let handle = PodmanHandle {
            id,
            container,
            podman: self.clone(),
        };

        match &handle.container.wait_strategy {
            Some(strategy) => self.wait_for(&handle, strategy)?,
            None => todo!(),
        };

        Ok(handle)
    }
}

pub struct PodmanHandle {
    id: String,
    container: Container,
    podman: Podman,
}

impl ContainerHandle for PodmanHandle {
    type LogType = SharedLogStream;

    fn stop(&mut self) {
        todo!()
    }

    fn log(&self) -> Self::LogType {
        let mut command = Command::new(Podman::BINARY);
        command.arg("logs").arg("-f").arg(&self.id);

        SharedLogStream { command }
    }

    fn container(&self) -> &Container {
        &self.container
    }
}

#[cfg(test)]
mod test {
    use crate::{
        client::{Client, ContainerHandle},
        container::Container,
    };

    use super::Podman;

    #[test]
    fn test_image() {
        // let client = Podman::new();
        // let pg_image = Postgres;
        // let pg_container = Container::from_image(pg_image);

        // let container = client.create(pg_container);

        // container.run();
    }
}
