use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use log::debug;

use crate::error::Error;

use super::LogStream;

pub struct SharedLogStream {
    pub command: Command,
}

impl SharedLogStream {
    fn new(command: Command) -> Self {
        Self { command }
    }
}

impl LogStream for SharedLogStream {
    fn stream(&mut self) -> Result<Box<dyn BufRead>, Error> {
        debug!("Running log command: {:?}", self.command);

        match self.command.stdout(Stdio::piped()).spawn() {
            Ok(mut child) => match child.stdout.take() {
                Some(stdout) => Ok(Box::new(BufReader::new(stdout))),
                None => Err(Error {
                    message: "Could not open stdout. Did the process exit already?".to_string(),
                }),
            },
            Err(_e) => Err(Error {
                message: "Error trying to run log command".to_string(),
            }),
        }
    }
}
