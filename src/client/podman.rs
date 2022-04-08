use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, ExitStatus, Stdio},
};

use log::debug;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;
use serde::Deserialize;

use crate::{container::*, error::Error};

use super::{shared::SharedLogStream, Client, ContainerHandle, LogStream};

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

    pub fn ps(&self) -> Result<Vec<PodmanContainer>, Error> {
        let json = self.run_and_wait_for_command(&mut self.build_ps_command())?;
        Ok(serde_json::from_str(&json)?)
    }

    fn build_ps_command(&self) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("ps").arg("--format").arg("json");

        command
    }

    fn build_rm_command(&self, podman_handle: &PodmanHandle) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("rm").arg("-f").arg(&podman_handle.id);

        command
    }

    fn build_stop_command(&self, podman_handle: &PodmanHandle) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("stop").arg(&podman_handle.id);

        command
    }

    fn build_run_command(&self, podman_handle: &PodmanHandle) -> Command {
        return self.build_run_command_from_container(&podman_handle.container);
    }

    fn build_run_command_from_container(&self, container: &Container) -> Command {
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

    fn run_and_wait_for_command(&self, command: &mut Command) -> Result<String, std::io::Error> {
        debug!("Run and wait for command: {:?}", command);

        let result = command
            .stdout(Stdio::piped()) // TODO fm - Sometimes podman asks the user for which repo to use. This is currently ignored.
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
                Err(_e) => return Err(Error {
                    message: "IO Error while reading log".to_string(),
                }),
            };
        }

        Err(Error {
            message: "Pattern defined for WaitStrategy could not be found in container output".to_string(),
        })
    }
}

impl Client for Podman {
    type ContainerHandle = PodmanHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle, Error> {
        let id = self
            .run_and_wait_for_command(&mut self.build_run_command_from_container(&container))?
            .trim()
            .to_string();

        let handle = PodmanHandle {
            id,
            container,
            podman: self.clone(),
        };

        match &handle.container.wait_strategy {
            Some(strategy) => self.wait_for(&handle, strategy)?,
            None => {}
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

    fn stop(&mut self) -> Result<(), Error> {
        self.podman
            .run_and_wait_for_command(&mut self.podman.build_stop_command(&self))?;

        Ok(())
    }

    fn log(&self) -> Self::LogType {
        let mut command = Command::new(Podman::BINARY);
        command.arg("logs").arg("-f").arg(&self.id);

        SharedLogStream { command }
    }

    fn container(&self) -> &Container {
        &self.container
    }

    fn start(&mut self) -> Result<(), Error> {
        self.podman
            .run_and_wait_for_command(&mut self.podman.build_run_command(self))?;

        Ok(())
    }

    fn rm(&mut self) -> Result<(), Error> {
        self.podman
            .run_and_wait_for_command(&mut self.podman.build_rm_command(self))?;

        Ok(())
    }
}

// impl Drop for PodmanHandle {
//     fn drop(&mut self) {
//         self.stop().unwrap();
//         self.rm().unwrap();
//     }
// }

#[cfg(test)]
mod test {
    use crate::{client::Client, container::Container, image::Image};

    use super::Podman;

    #[test]
    fn test_scope() {
        {
            let handle = Podman::new()
                .create(Container::from_image(Image::from_name("nginx")))
                .unwrap();
        }
    }
}
