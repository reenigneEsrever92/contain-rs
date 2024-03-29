use std::{
    io::BufRead,
    process::{Command, Output, Stdio},
    thread,
    time::Duration,
};

use regex::Regex;
use tracing::*;

use crate::{
    container::{Container, Volume, WaitStrategy},
    error::{ContainerResult, ContainersError},
    rt::{ContainerStatus, DetailedContainerInfo},
};

use super::{Client, Log};

pub fn run_and_wait_for_command(command: &mut Command) -> ContainerResult<String> {
    let output = try_run_and_wait_for_command(command)?;

    if let Some(0) = output.status.code() {
        Ok(String::from_utf8(output.stdout).unwrap())
    } else {
        Err(ContainersError::CommandError(output))
    }
}

#[instrument(skip_all)]
pub fn try_run_and_wait_for_command(command: &mut Command) -> ContainerResult<Output> {
    debug!(?command, "Running command");

    let child = command
        .stdout(Stdio::piped()) // TODO fm - Sometimes podman asks the user for which repo to use. This is currently ignored.
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    Ok(child.wait_with_output()?)
}

pub fn build_log_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    command.arg("logs").arg("-f").arg(&container.name)
}

pub fn build_rm_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    command.arg("rm").arg("-f").arg(&container.name)
}

pub fn build_stop_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    command.arg("stop").arg(&container.name)
}

pub fn build_run_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    add_run_args(command);
    add_name_arg(command, container);
    add_env_var_args(command, container);
    add_volume_args(command, container);
    add_export_ports_args(command, container);
    add_health_check_args(command, container);
    add_image_arg(command, container);
    add_command_arg(command, container);

    command
}

fn add_volume_args<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    let folded = container
        .volumes
        .iter()
        .fold(command, |c: &mut Command, volume| match volume {
            Volume::Mount {
                host_path,
                mount_point,
            } => c.arg("-v").arg(format!("{host_path}:{mount_point}")),
            Volume::Named { name, mount_point } => c.arg("-v").arg(format!("{name}:{mount_point}")),
        });

    folded
}

fn add_command_arg<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    let folded = container
        .command
        .iter()
        .fold(command, |c: &mut Command, arg| c.arg(arg));

    folded
}

fn add_name_arg<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    command.arg("--name").arg(&container.name)
}

pub fn build_inspect_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    command.arg("inspect").arg(&container.name)
}

fn add_run_args(command: &mut Command) {
    command.arg("run").arg("-d");
}

fn add_env_var_args(command: &mut Command, container: &Container) {
    container.env_vars.iter().for_each(|env_var| {
        command
            .arg("-e")
            .arg(format!("{}={}", env_var.key, env_var.value));
    });
}

fn add_health_check_args(command: &mut Command, container: &Container) {
    if let Some(check) = &container.health_check {
        command.arg("--health-cmd").arg(&check.command);

        if let Some(start_period) = check.start_period {
            command.arg(format!("--health-start-period={}s", start_period.as_secs()));
        }

        if let Some(interval) = check.interval {
            command.arg(format!("--health-interval={}s", interval.as_secs()));
        }

        if let Some(timeout) = check.timeout {
            command.arg(format!("--health-timeout={}s", timeout.as_secs()));
        }

        if let Some(retries) = check.retries {
            command.arg(format!("--health-retries={}", retries));
        }
    }
}

fn add_image_arg(command: &mut Command, container: &Container) {
    command.arg(String::from(&container.image));
}

fn add_export_ports_args(command: &mut Command, container: &Container) {
    container.port_mappings.iter().for_each(|port_mapping| {
        command.arg(format!(
            "-p{}:{}",
            port_mapping.source.number, port_mapping.target.number
        ));
    })
}

#[instrument(skip_all)]
pub fn inspect<C: Client>(
    client: &C,
    container: &Container,
) -> ContainerResult<Option<DetailedContainerInfo>> {
    let mut cmd = client.command();

    build_inspect_command(&mut cmd, container);

    let output = try_run_and_wait_for_command(&mut cmd)?;

    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();

    match output.status.code() {
        Some(0) => {
            let container_infos: Vec<DetailedContainerInfo> = serde_json::from_str(&stdout)?;

            debug!(?container_infos, "Inspect container");

            match container_infos.get(0) {
                Some(info) => Ok(Some(info.to_owned())),
                None => Ok(None),
            }
        }
        _ => {
            if stderr.to_uppercase().contains("NO SUCH OBJECT") {
                Ok(None)
            } else {
                Err(ContainersError::CommandError(output))
            }
        }
    }
}

#[instrument(skip_all)]
pub fn do_log<C: Client>(client: &C, container: &Container) -> ContainerResult<Log> {
    let mut cmd = client.command();

    build_log_command(&mut cmd, container);

    let (reader, writer) = os_pipe::pipe()?;

    // redirect all process out to that single pipe
    let cmd = cmd
        .stdout(writer.try_clone().unwrap())
        .stderr(writer)
        .spawn()?;

    debug!(?cmd, "Reading log");

    Ok(Log { reader })
}

pub fn wait_for<C: Client>(client: &C, container: &Container) -> ContainerResult<()> {
    let result = match &container.wait_strategy {
        Some(strategy) => match strategy {
            WaitStrategy::LogMessage { pattern } => {
                wait_for_log(client, container, strategy, pattern)
            }
            WaitStrategy::HealthCheck => Ok(wait_for_health_check(client, container)?),
            WaitStrategy::WaitTime { duration } => Ok(wait_for_time(duration.to_owned())?),
        },
        None => Ok(()),
    };

    thread::sleep(container.additional_wait_period);

    result
}

fn wait_for_time(duration: Duration) -> ContainerResult<()> {
    thread::sleep(duration);
    Ok(())
}

fn wait_for_log<C: Client>(
    client: &C,
    container: &Container,
    wait_strategy: &WaitStrategy,
    pattern: &Regex,
) -> ContainerResult<()> {
    let log = do_log(client, container)?;

    do_wait_for_log(pattern, container, wait_strategy, log)?;

    Ok(())
}

#[instrument(skip_all)]
fn do_wait_for_log(
    pattern: &Regex,
    container: &Container,
    wait_strategy: &WaitStrategy,
    mut log: Log,
) -> ContainerResult<()> {
    debug!(?pattern, "Searching log");
    for result in log.stream().lines() {
        let line = result?;
        debug!(?pattern, ?line, "Searching for pattern");
        if pattern.is_match(&line) {
            debug!(?line, ?pattern, "Found pattern");
            return Ok(());
        }
    }

    Err(ContainersError::ContainerWaitFailed {
        container_name: container.name.clone(),
        wait_strategy: wait_strategy.clone(),
    })
}

#[instrument(skip_all)]
fn wait_for_health_check<C: Client>(client: &C, container: &Container) -> ContainerResult<()> {
    loop {
        debug!("Checking health for {}", &container.name);

        match inspect(client, container)? {
            Some(info) => {
                if let Some(health) = info.state.health {
                    match health.status {
                        ContainerStatus::Healthy => return Ok(()),
                        ContainerStatus::Starting => thread::sleep(Duration::from_millis(200)),
                        _ => {
                            return Err(ContainersError::ContainerStatusError {
                                status: health.status,
                            })
                        }
                    }
                }
            }
            None => {
                return Err(ContainersError::ContainerNotExists {
                    container_name: container.name.clone(),
                })
            }
        }
    }
}
