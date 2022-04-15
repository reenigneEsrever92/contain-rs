use std::{
    io::{BufRead, BufReader},
    process::{Command, Output, Stdio},
    thread,
    time::Duration,
};

use log::debug;
use regex::Regex;

use crate::{
    container::{Container, HealthCheck, WaitStrategy},
    error::{Context, ErrorType, Result},
};

use super::{Handle, Log};

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

    debug!("Command result: {:?}", result);

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

pub fn build_ps_command<'a>(command: &'a mut Command) -> &'a Command {
    command.arg("ps").arg("--format").arg("json")
}

pub fn build_stop_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    command.arg("stop").arg(&container.name)
}

pub fn build_run_command<'a>(command: &'a mut Command, container: &Container) -> &'a Command {
    add_run_args(command);
    add_name_arg(command, container);
    add_env_var_args(command, container);
    add_export_ports_args(command, container);
    add_image_arg(command, container);
    add_wait_strategy_args(command, container);

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

pub fn do_log(log_command: &mut Command) -> Result<Log> {
    // when containers are just in the making accessing logs to early can result in errors
    // TODO - fm maybe we can somehow get rid of it, but I wouldn't know how
    thread::sleep(Duration::from_millis(2000)); 
    match log_command
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => Ok(Log { child }),
        Err(e) => Err(Context::new()
            .source(e)
            .info("message", "Could not spawn log command")
            .into_error(ErrorType::LogError)),
    }
}

pub fn wait_for(mut command: Command, container: &Container) -> Result<()> {
    match &container.wait_strategy {
        Some(strategy) => match strategy {
            WaitStrategy::LogMessage { pattern } => {
                build_log_command(&mut command, container);

                match do_log(&mut command) {
                    Ok(log) => {
                        let result = wait_for_log(&pattern, log);

                        result
                    },
                    Err(e) => Err(Context::new()
                        .source(e)
                        .info("message", "Waiting for log output failed")
                        .into_error(ErrorType::LogError)),
                }
            }
            WaitStrategy::HealthCheck { check: _ } => todo!(),
        },
        None => Ok(()),
    }
}

pub fn wait_for_log(pattern: &Regex, mut log: Log) -> Result<()> {
    match log.stream() {
        Some(read) => {
            for line in read.lines() {
                match line {
                    Ok(line) => {
                        debug!("Searching for LogMessage pattern: {pattern}, in: {line}");
                        if pattern.is_match(&line) {
                            debug!("Found pattern in line: {line}");
                            // wait a little after reading the log message just to be sure the container really is ready
                            thread::sleep(Duration::from_millis(2000));
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

#[allow(dead_code)]
fn wait_for_health_check<T: Handle>(_handle: &T) -> Result<()> {
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
