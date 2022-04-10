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
        build_log_command, build_ps_command, build_rm_command, build_run_command,
        build_run_command_from_container, build_stop_command, do_log, run_and_wait_for_command,
        wait_for_log,
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

        build_ps_command(& mut command);

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

    fn wait_for(&self, handle: &PodmanHandle, strategy: &WaitStrategy) -> Result<()> {
        let mut command = self.build_command();

        build_log_command(& mut command, &handle.instance);

        match strategy {
            WaitStrategy::LogMessage { pattern } => match do_log(command) {
                Ok(log) => wait_for_log(&handle.instance, &pattern, log),
                Err(e) => Err(Context::new()
                    .source(e)
                    .info("message", "Waiting for log output failed")
                    .into_error(ErrorType::LogError)),
            },
            WaitStrategy::HealthCheck { check: _ } => self.wait_for_health_check(handle),
        }
    }

    fn wait_for_health_check(&self, handle: &PodmanHandle) -> Result<()> {
        thread::sleep(Duration::from_secs(10));

        match run_and_wait_for_command(self.build_health_check_command(&handle.instance)) {
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
        let mut command = self.build_command();

        build_run_command_from_container(& mut command, &container);

        let id = run_and_wait_for_command(command)?.trim().to_string();

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
        let mut command = self.podman.build_command();

        build_stop_command(& mut command, &self.instance);
        run_and_wait_for_command(command)?;

        Ok(())
    }

    fn log(&self) -> Result<Box<dyn BufRead>> {
        let mut command = self.podman.build_command();

        build_log_command(& mut command, &self.instance);

        do_log(command)
    }

    fn container(&self) -> &Container {
        &self.instance.container
    }

    fn start(&mut self) -> Result<()> {
        let mut command = self.podman.build_command();

        build_run_command(& mut command, &self.instance);
        run_and_wait_for_command(command)?;

        Ok(())
    }

    fn rm(&mut self) -> Result<()> {
        let mut command = self.podman.build_command();

        build_rm_command(& mut command, &self.instance);
        run_and_wait_for_command(command)?;

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
