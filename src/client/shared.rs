use std::{process::{Command, Stdio}, io::{BufRead, BufReader}};

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
        match self.command.stdout(Stdio::piped()).spawn() {
            Ok(mut child) => match child.stdout.take() {
                Some(stdout) => Ok(Box::new(BufReader::new(stdout))),
                None => Err(Error::LogError {
                    message: "Could not open stdout. Did the process exit already?".to_string(),
                }),
            },
            Err(e) => todo!(),
        }
    }
}