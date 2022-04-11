/// 
/// This module provides postgres image utilities.
/// 
/// ## Usage:
/// 
/// 

use std::str::FromStr;

use regex::Regex;

use crate::container::{Container, WaitStrategy};

use super::Image;

pub struct Postgres {
    password: String,
}

impl Postgres {
    const IMAGE: &'static str = "docker.io/library/postgres";

    pub fn default() -> Self {
        Self {
            password: Postgres::IMAGE.to_string(),
        }
    }

    pub fn with_password(mut self, password: &'static str) -> Self {
        self.password = password.to_string();
        self
    }

    pub fn container(self) -> Container {
        Container::from_image(Image::from_name(Postgres::IMAGE))
            .wait_for(WaitStrategy::LogMessage {
                pattern: Regex::from_str("database system is ready to accept connections").unwrap(),
            })
            .env_var(("POSTGRES_PASSWORD", self.password))
    }
}

#[cfg(test)]
mod test {
    use crate::client::{podman::Podman, Client, ContainerHandle};

    use super::Postgres;

    #[test]
    fn test_default() {
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let client = Podman::new();
        let container = Postgres::default().with_password("password").container();

        let mut handle = client.create(container);

        handle.run().unwrap();
        handle.stop().unwrap();
    }
}
