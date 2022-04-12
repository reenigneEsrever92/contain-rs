use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use log::debug;
use regex::Regex;

use crate::{
    rt::ContainerInstance,
    container::{Container, HealthCheck, WaitStrategy},
    error::{Context, ErrorType, Result},
};

use super::ContainerHandle;

pub fn run_and_wait_for_command(mut command: Command) -> Result<String> {
    debug!("Run and wait for command: {:?}", command);

    let child = command
        .stdout(Stdio::piped()) // TODO fm - Sometimes podman asks the user for which repo to use. This is currently ignored.
        .stderr(Stdio::piped())
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
                    .info("stderror", &String::from_utf8(output.stderr).unwrap())
                    .into_error(ErrorType::CommandError))
            }
        }
        Err(e) => Err(Context::new()
            .info("reason", "Io error while getting process output")
            .into_error(ErrorType::Unrecoverable)),
    }
}

pub fn build_log_command<'a>(
    command: &'a mut Command,
    id: &str,
) -> &'a Command {
    command.arg("logs").arg("-f").arg(id)
}

pub fn build_rm_command<'a>(command: &'a mut Command, id: &str) -> &'a Command {
    command.arg("rm").arg("-f").arg(id)
}

pub fn build_ps_command<'a>(command: &'a mut Command) -> &'a Command {
    command.arg("ps").arg("--format").arg("json")
}

pub fn build_stop_command<'a>(
    command: &'a mut Command,
    id: &str,
) -> &'a Command {
    command.arg("stop").arg(id)
}

pub fn build_run_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    add_run_args(command);
    add_env_var_args(command, container);
    add_export_ports_args(command, container);
    add_image_arg(command, container);
    add_wait_strategy_args(command, container);

    command
}

pub fn build_inspect_command<'a>(command: &'a mut Command, instance: &ContainerInstance) -> &'a Command {
    command.arg("inspect").arg(&instance.id)
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

fn add_wait_strategy_args(command: &mut Command, container: &Container) {
    match &container.wait_strategy {
        Some(strategy) => match strategy {
            WaitStrategy::LogMessage { pattern: _ } => {}
            WaitStrategy::HealthCheck { check } => add_health_check_args(command, check),
        },
        None => {}
    }
}

fn add_health_check_args(command: &mut Command, check: &HealthCheck) {
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

pub fn wait_for_log(
    instance: &ContainerInstance,
    pattern: &Regex,
    log: Box<dyn BufRead>,
) -> Result<()> {
    for line in log.lines() {
        match line {
            Ok(line) => {
                debug!("Searching for LogMessage pattern: {pattern}, in: {line}");
                if pattern.is_match(&line) {
                    debug!("Found pattern in line: {line}");
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

    Err(Context::new()
        .info(
            "reason",
            "Log has been closed before message could be found",
        )
        .into_error(ErrorType::WaitError))
}

pub fn do_log(mut log_command: Command) -> Result<Box<dyn BufRead>> {
    match log_command.stdout(Stdio::piped()).spawn() {
        Ok(mut child) => match child.stdout.take() {
            Some(stdout) => Ok(Box::new(BufReader::new(stdout))),
            None => Err(Context::new()
                .info("message", "Stoud could not be opened")
                .into_error(ErrorType::LogError)),
        },
        Err(e) => Err(Context::new()
            .source(e)
            .info("message", "Could not spawn log command")
            .into_error(ErrorType::LogError)),
    }
}

pub fn wait_for(
    mut command: Command,
    instance: &ContainerInstance,
    strategy: &WaitStrategy,
) -> Result<()> {
    build_log_command(&mut command, &instance.id);

    match strategy {
        WaitStrategy::LogMessage { pattern } => match do_log(command) {
            Ok(log) => wait_for_log(instance, &pattern, log),
            Err(e) => Err(Context::new()
                .source(e)
                .info("message", "Waiting for log output failed")
                .into_error(ErrorType::LogError)),
        },
        WaitStrategy::HealthCheck { check: _ } => todo!(),
    }
}

fn wait_for_health_check<T: ContainerHandle>(handle: &T) -> Result<()> {
    todo!()
    // thread::sleep(Duration::from_secs(10));

    // match run_and_wait_for_command(self.build_health_check_command(&handle.instance)) {
    //     Ok(_) => Ok(()),
    //     Err(e) => Err(Context::new()
    //         .info("reason", "Healthcheck failed")
    //         .source(e)
    //         .into_error(ErrorType::WaitError)),
    // }
}
