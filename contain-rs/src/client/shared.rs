use std::{
    io::BufRead,
    process::{Command, Output, Stdio},
    thread,
    time::Duration,
};

use log::{debug, trace};
use regex::Regex;

use crate::{
    container::{Container, WaitStrategy},
    error::{Context, ErrorType, Result},
    rt::{ContainerStatus, DetailedContainerInfo},
};

use super::{Client, Log};

pub fn run_and_wait_for_command_infallible(command: &mut Command) -> Result<String> {
    match run_and_wait_for_command(command) {
        Ok(output) => {
            if let Some(0) = output.status.code() {
                Ok(String::from_utf8(output.stdout).unwrap())
            } else {
                Err(Context::new()
                    .info("message", "Non zero exit-code")
                    .info("exit-code", &output.status.code().unwrap())
                    .info("command", command)
                    .info("stderror", &String::from_utf8(output.stderr).unwrap())
                    .into_error(ErrorType::CommandError))
            }
        }
        Err(e) => Err(Context::new()
            .info("message", "Io error while getting process output")
            .source(e)
            .into_error(ErrorType::Unrecoverable)),
    }
}

pub fn run_and_wait_for_command(command: &mut Command) -> Result<Output> {
    debug!("Run and wait for command: {:?}", command);

    let child = command
        .stdout(Stdio::piped()) // TODO fm - Sometimes podman asks the user for which repo to use. This is currently ignored.
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let result = child.wait_with_output();

    trace!("Command result: {:?}", result);

    match result {
        Ok(output) => Ok(output),
        Err(e) => Err(Context::new()
            .info("message", "Io error while getting process output")
            .source(e)
            .into_error(ErrorType::Unrecoverable)),
    }
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
    add_export_ports_args(command, container);
    add_health_check_args(command, container);
    add_image_arg(command, container);

    command
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

pub fn inspect<C: Client>(
    client: &C,
    container: &Container,
) -> Result<Option<DetailedContainerInfo>> {
    let mut cmd = client.command();

    build_inspect_command(&mut cmd, container);

    let output = run_and_wait_for_command(&mut cmd)?;

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    match output.status.code() {
        Some(0) => {
            let container_infos: Vec<DetailedContainerInfo> = serde_json::from_str(&stdout)
                .map_err(|e| {
                    Context::new()
                        .source(e)
                        .info("message", "could not parse inspect output")
                        .info("json", &stdout)
                        .into_error(ErrorType::JsonError)
                })?;

            debug!("Inspect json: {}", serde_json::to_string_pretty(&container_infos).unwrap());

            match container_infos.get(0) {
                Some(info) => Ok(Some(info.to_owned())),
                None => Ok(None),
            }
        }
        _ => {
            if stderr.to_uppercase().contains("NO SUCH OBJECT") {
                Ok(None)
            } else {
                Err(Context::new()
                    .info("message", "Unexpected error while inspecting container")
                    .info("stderr", &stderr)
                    .info("stdout", &stdout)
                    .into_error(ErrorType::CommandError))
            }
        }
    }
}

pub fn do_log<C: Client>(client: &C, container: &Container) -> Result<Log> {
    let mut cmd = client.command();

    build_log_command(&mut cmd, container);

    match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(child) => Ok(Log { child }),
        Err(e) => Err(Context::new()
            .source(e)
            .info("message", "Could not spawn log command")
            .into_error(ErrorType::LogError)),
    }
}

pub fn wait_for<C: Client>(client: &C, container: &Container) -> Result<()> {
    let result = match &container.wait_strategy {
        Some(strategy) => match strategy {
            WaitStrategy::LogMessage { pattern } => wait_for_log(client, container, pattern),
            WaitStrategy::HealthCheck => wait_for_health_check(client, container),
        },
        None => Ok(()),
    };

    thread::sleep(container.additional_wait_period);

    result
}

fn wait_for_log<C: Client>(client: &C, container: &Container, pattern: &Regex) -> Result<()> {
    match do_log(client, container) {
        Ok(log) => do_wait_for_log(pattern, log),
        Err(e) => Err(Context::new()
            .source(e)
            .info("message", "Waiting for log output failed")
            .into_error(ErrorType::LogError)),
    }
}

fn do_wait_for_log(pattern: &Regex, mut log: Log) -> Result<()> {
    match log.stream() {
        Some(read) => {
            for line in read.lines() {
                match line {
                    Ok(line) => {
                        debug!("Searching for LogMessage pattern: {pattern}, in: {line}");
                        if pattern.is_match(&line) {
                            debug!("Found pattern in line: {line}");
                            // wait a little after reading the log message just to be sure the container really is ready
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        return Err(Context::new()
                            .source(e)
                            .info("reason", "Could not read from log")
                            .into_error(ErrorType::LogError))
                    }
                }
            }
        }
        None => {}
    }

    Err(Context::new()
        .info(
            "reason",
            "Log has been closed before message could be found",
        )
        .into_error(ErrorType::WaitError))
}

fn wait_for_health_check<C: Client>(client: &C, container: &Container) -> Result<()> {
    loop {
        debug!("Checking health for {}", &container.name);

        match inspect(client, container)? {
            Some(info) => {
                if let Some(health) = info.state.health {
                    match health.status {
                        ContainerStatus::Healthy => return Ok(()),
                        ContainerStatus::Starting => thread::sleep(Duration::from_millis(200)),
                        _ => {
                            return Err(Context::new()
                                .info("message", "Invalid container status")
                                .into_error(ErrorType::WaitError))
                        }
                    }
                }
            }
            None => {
                return Err(Context::new()
                    .info("message", "Container not running")
                    .into_error(ErrorType::WaitError))
            }
        }
    }
}
