//!
//! This module contains the traits that represent the bare functiionality of a client.
//!
//! See [Podman] for usage of the podman client implementation.
//!

use std::{
    io::{BufRead, BufReader},
    process::Command,
};

use os_pipe::PipeReader;

use crate::{
    container::{Container, IntoContainer},
    error::ContainerResult,
    rt::DetailedContainerInfo,
};

pub mod docker;
pub mod podman;
pub mod shared;

///
/// The client Trait represents a way to access a client.
///
/// It's implemented by the [Podman] struct for example. And will be for docker as well.
/// If you do that for any specific Type you get [`handles`](Handle) for free.
///
///
pub trait Client: Clone {
    type ClientType: Client;

    fn command(&self) -> Command;
    fn create<C: IntoContainer>(&self, container: C) -> ContainerHandle<Self::ClientType>;
    fn run(&self, container: &Container) -> ContainerResult<()>;
    fn stop(&self, container: &Container) -> ContainerResult<()>;
    fn rm(&self, container: &Container) -> ContainerResult<()>;
    fn log(&self, container: &Container) -> ContainerResult<Option<Log>>;
    fn inspect(&self, container: &Container) -> ContainerResult<Option<DetailedContainerInfo>>;
    fn exists(&self, container: &Container) -> ContainerResult<bool>;
    fn runs(&self, container: &Container) -> ContainerResult<bool>;
    fn wait(&self, container: &Container) -> ContainerResult<()>;
}

///
/// A handle is a way to interact with a container.
///
/// When you create a container using a [Client] it will return one of these.
/// The handle automatically stops and removes the container, when it goes out of scope.
///
pub trait Handle {
    fn run(&self) -> ContainerResult<()>;
    fn wait(&self) -> ContainerResult<()>;
    fn run_and_wait(&self) -> ContainerResult<()>;
    fn stop(&self) -> ContainerResult<()>;
    fn rm(&self) -> ContainerResult<()>;
    fn log(&self) -> ContainerResult<Option<Log>>;
    fn container(&self) -> &Container;
    fn is_running(&self) -> ContainerResult<bool>;
    fn exists(&self) -> ContainerResult<bool>;
}

pub struct Log {
    pub reader: PipeReader,
}

impl Log {
    fn stream(&mut self) -> impl BufRead {
        BufReader::new(self.reader.try_clone().unwrap())
    }
}

pub struct ContainerHandle<T: Client> {
    client: T,
    container: Container,
}

impl<T: Client> Handle for ContainerHandle<T> {
    fn run(&self) -> ContainerResult<()> {
        if !self.is_running()? {
            self.client.run(&self.container)?;
        }

        Ok(())
    }

    fn wait(&self) -> ContainerResult<()> {
        self.client.wait(&self.container)?;

        Ok(())
    }

    fn run_and_wait(&self) -> ContainerResult<()> {
        if !self.is_running()? {
            self.run()?;
            self.wait()?;
        }

        Ok(())
    }

    fn stop(&self) -> ContainerResult<()> {
        if self.is_running()? {
            self.client.stop(&self.container)?
        }

        Ok(())
    }

    fn rm(&self) -> ContainerResult<()> {
        self.stop()?;

        if self.exists()? {
            self.client.rm(&self.container)?
        }

        Ok(())
    }

    fn log(&self) -> ContainerResult<Option<Log>> {
        if self.is_running()? {
            Ok(self.client.log(&self.container)?)
        } else {
            Ok(None)
        }
    }

    fn container(&self) -> &Container {
        &self.container
    }

    fn is_running(&self) -> ContainerResult<bool> {
        Ok(self.exists()? && self.client.runs(&self.container)?)
    }

    fn exists(&self) -> ContainerResult<bool> {
        self.client.exists(&self.container)
    }
}

impl<T: Client> Drop for ContainerHandle<T> {
    fn drop(&mut self) {
        self.rm().unwrap();
    }
}
