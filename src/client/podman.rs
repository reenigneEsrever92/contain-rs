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

use super::{Client, ContainerHandle};

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
        let result = self.run_and_wait_for_command(self.build_ps_command());

        match result {
            Ok(output) => match serde_json::from_str(&output) {
                Ok(vec) => Ok(vec),
                Err(e) => Err(Context::new()
                    .info("reason", "could not parse json")
                    .info("json", &output)
                    .into_error(ErrorType::PsError)),
            },
            Err(e) => Err(Context::new().source(e).into_error(ErrorType::PsError)),
        }
    }

    fn do_log(&self, handle: &PodmanHandle) -> Result<Box<dyn BufRead>> {
        let mut command = Command::new(Podman::BINARY);
        command.arg("logs").arg("-f").arg(&handle.instance.id);

        match command.stdout(Stdio::piped()).spawn() {
            Ok(mut child) => match child.stdout.take() {
                Some(stdout) => Ok(Box::new(BufReader::new(stdout))),
                None => Err(Context::new()
                    .info("reason", "Stoud could not be opened")
                    .into_error(ErrorType::LogError)),
            },
            Err(e) => Err(Context::new()
                .source(e)
                .span_trace(SpanTrace::capture())
                .info("reason", "Could not spawn log command")
                .into_error(ErrorType::LogError)),
        }
    }

    fn build_ps_command(&self) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("ps").arg("--format").arg("json");

        command
    }

    fn build_rm_command(&self, podman_handle: &PodmanHandle) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("rm").arg("-f").arg(&podman_handle.instance.id);

        command
    }

    fn build_stop_command(&self, podman_handle: &PodmanHandle) -> Command {
        let mut command = Command::new(Self::BINARY);

        command.arg("stop").arg(&podman_handle.instance.id);

        command
    }

    fn build_run_command(&self, podman_handle: &PodmanHandle) -> Command {
        return self.build_run_command_from_container(&podman_handle.instance.container);
    }

    fn build_run_command_from_container(&self, container: &Container) -> Command {
        let mut command = Command::new(Self::BINARY);

        self.add_run_args(&mut command);
        self.add_env_var_args(&mut command, container);
        self.add_image_arg(&mut command, container);
        self.add_wait_strategy_args(&mut command, container);

        command
    }

    fn build_health_check_command(&self, handle: &PodmanHandle) -> Command {
        let mut command = Command::new(Self::BINARY);

        command
            .arg("healthcheck")
            .arg("run")
            .arg(&handle.instance.id);

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

    fn add_wait_strategy_args(&self, command: &mut Command, container: &Container) {
        match &container.wait_strategy {
            Some(strategy) => match strategy {
                WaitStrategy::LogMessage { pattern: _ } => {}
                WaitStrategy::HealthCheck { check } => self.add_health_check_args(command, check),
            },
            None => {}
        }
    }

    fn add_health_check_args(&self, command: &mut Command, check: &HealthCheck) {
        command
            .arg("--healthcheck-command")
            .arg(format!("CMD-SHELL {}", check.command));

        if let Some(retries) = check.retries {
            command
                .arg("--healthcheck-retries")
                .arg(retries.to_string());
        }

        if let Some(duration) = check.interval {
            command
                .arg("--healthcheck-interval")
                .arg(format!("{}s", duration.as_secs()));
        }

        if let Some(duration) = check.start_period {
            command
                .arg("--healthcheck-start-period")
                .arg(format!("{}s", duration.as_secs()));
        }
    }

    fn add_image_arg(&self, command: &mut Command, container: &Container) {
        command.arg(String::from(&container.image));
    }

    fn run_and_wait_for_command(&self, mut command: Command) -> Result<String> {
        debug!("Run and wait for command: {:?}", command);

        let child = command
            .stdout(Stdio::piped()) // TODO fm - Sometimes podman asks the user for which repo to use. This is currently ignored.
            .spawn()
            .unwrap();

        let result = child.wait_with_output();

        debug!("Command result: {:?}", result);

        match result {
            Ok(output) => {
                if let Some(0) = output.status.code() {
                    Ok(String::from_utf8(output.stdout).unwrap())
                } else {
                    Err(Context::new()
                        .info("reason", "Non zero exit-code")
                        .info("exit-code", &output.status.code().unwrap())
                        .info("command", &command)
                        .into_error(ErrorType::CommandError))
                }
            }
            Err(e) => Err(Context::new()
                .info("reason", "Io error while getting process output")
                .into_error(ErrorType::Unrecoverable)),
        }
    }

    fn wait_for(&self, handle: &PodmanHandle, strategy: &WaitStrategy) -> Result<()> {
        match strategy {
            WaitStrategy::LogMessage { pattern } => self.wait_for_log(handle, &pattern),
            WaitStrategy::HealthCheck { check: _ } => self.wait_for_health_check(handle),
        }
    }

    fn wait_for_log(&self, handle: &PodmanHandle, pattern: &Regex) -> Result<()> {
        let stream = self.do_log(handle)?;

        for line in stream.lines() {
            match line {
                Ok(line) => {
                    debug!("Searching for LogMessage pattern: {pattern}, in: {line}");
                    if pattern.is_match(&line) {
                        debug!("Found pattern in line: {line}");
                        return Ok(());
                    }
                }
                Err(e) => todo!(),
            }
        }

        Err(Context::new()
            .info(
                "reason",
                "Log has been closed before message could be found",
            )
            .into_error(ErrorType::WaitError))
    }

    fn wait_for_health_check(&self, handle: &PodmanHandle) -> Result<()> {
        thread::sleep(Duration::from_secs(10));

        match self.run_and_wait_for_command(self.build_health_check_command(handle)) {
            Ok(_) => Ok(()),
            Err(e) => Err(Context::new()
                .info("reason", "Healthcheck failed")
                .source(e)
                .into_error(ErrorType::WaitError)),
        }
    }
}

impl Client for Podman {
    type ContainerHandle = PodmanHandle;

    fn create(&self, container: Container) -> Result<Self::ContainerHandle> {
        let id = self
            .run_and_wait_for_command(self.build_run_command_from_container(&container))?
            .trim()
            .to_string();

        let handle = PodmanHandle {
            instance: ContainerInstance::new(id, container),
            podman: self.clone(),
        };

        match &handle.instance.container.wait_strategy {
            Some(strategy) => self.wait_for(&handle, strategy)?,
            None => {}
        };

        Ok(handle)
    }
}

pub struct PodmanHandle {
    instance: ContainerInstance,
    podman: Podman,
}

impl ContainerHandle for PodmanHandle {
    fn stop(&mut self) -> Result<()> {
        self.podman
            .run_and_wait_for_command(self.podman.build_stop_command(&self))?;

        Ok(())
    }

    fn log(&self) -> Result<Box<dyn BufRead>> {
        self.podman.do_log(self)
    }

    fn container(&self) -> &Container {
        &self.instance.container
    }

    fn start(&mut self) -> Result<()> {
        self.podman
            .run_and_wait_for_command(self.podman.build_run_command(self))?;

        Ok(())
    }

    fn rm(&mut self) -> Result<()> {
        self.podman
            .run_and_wait_for_command(self.podman.build_rm_command(self))?;

        Ok(())
    }
}

impl Drop for PodmanHandle {
    fn drop(&mut self) {
        self.stop().unwrap();
        self.rm().unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::{
        client::Client,
        container::{Container, HealthCheck},
        image::Image,
    };

    use super::Podman;

    #[test]
    fn test_scope() {
        {
            let handle = Podman::new()
                .create(Container::from_image(Image::from_name("nginx")))
                .unwrap();
        }
    }

    #[test]
    fn test_wait_for_healthcheck() {
        // pretty_env_logger::formatted_timed_builder()
        //     .filter_level(log::LevelFilter::Debug)
        //     .init();

        // let handle = Podman::new()
        //     .create(Container::from_image(Image::from_name("nginx")).wait_for(
        //         crate::container::WaitStrategy::HealthCheck {
        //             check: HealthCheck::new("curl http://localhost || exit 1".to_string()),
        //         },
        //     ))
        //     .unwrap();
    }
}
