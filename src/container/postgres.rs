///
/// This module provides postgres image utilities.
///
/// ## Usage:
///
///
use std::{time::Duration};

use crate::container::{Container, WaitStrategy};

use super::{HealthCheck, Image};

pub struct Postgres {
    db: Option<String>,
    user: Option<String>,
    password: String,
}

impl Postgres {
    const IMAGE: &'static str = "docker.io/library/postgres";

    pub fn default() -> Self {
        Self {
            db: None,
            user: None,
            password: Postgres::IMAGE.to_string(),
        }
    }

    pub fn with_password(mut self, password: &str) -> Self {
        self.password = password.to_string();
        self
    }

    pub fn with_user(mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self
    }

    pub fn with_db(mut self, db: &str) -> Self {
        self.db = Some(db.to_string());
        self
    }

    pub fn container(self) -> Container {
        let mut container = Container::from_image(Image::from_name(Postgres::IMAGE))
            .health_check(
                HealthCheck::new("pg_isready")
            )
            .additional_wait_period(Duration::from_secs(2))
            .wait_for(WaitStrategy::HealthCheck)
            .env_var(("POSTGRES_PASSWORD", self.password));

        if let Some(db) = self.db {
            container = container.env_var(("POSTGRES_DB", db));
        }

        if let Some(user) = self.user {
            container = container.env_var(("POSTGRES_USER", user));
        }

        container
    }
}

#[cfg(test)]
mod test {
    use diesel::{Connection, PgConnection};

    use crate::client::{podman::Podman, Client, Handle};

    use super::Postgres;

    #[test]
    fn test_default() {
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

        let client = Podman::new();
        let container = Postgres::default()
            .with_db("test-db")
            .with_user("pg-user")
            .with_password("password")
            .container()
            .map_port(5432, 5432);

        let handle = client.create(container);

        handle.run();

        let connection =
            PgConnection::establish("postgres://pg-user:password@localhost:5432/test-db").unwrap();

        assert!(connection
            .execute(
                "CREATE TABLE posts (
                    id SERIAL PRIMARY KEY,
                    title VARCHAR NOT NULL,
                    body TEXT NOT NULL,
                    published BOOLEAN NOT NULL DEFAULT 'f'
                )"
            )
            .is_ok());

        handle.stop();
    }
}
