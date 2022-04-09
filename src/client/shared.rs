use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use log::debug;

use crate::{error::{Context, ContainersError, Result}, container::ContainerInstance};

use super::LogStream;

