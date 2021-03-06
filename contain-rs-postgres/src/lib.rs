// ///
// /// This module provides postgres image utilities.
// ///
// /// ## Usage:
// ///
// ///
// use std::time::Duration;

// use contain_rs::container::{Container, HealthCheck, IntoContainer, WaitStrategy};
// use contain_rs_builder::{declare, image};
// use contain_rs_macro::container;

// #[derive(Default)]
// #[container(declare(image("docker.io/library/postgres"), []))]
// struct PostgresBase;

// pub struct Postgres {
//     db: Option<String>,
//     user: Option<String>,
//     password: String,
// }

// impl Postgres {
//     const IMAGE: &'static str = "docker.io/library/postgres";

//     pub fn default() -> Self {
//         Self {
//             db: None,
//             user: None,
//             password: Postgres::IMAGE.to_string(),
//         }
//     }

//     pub fn with_password(mut self, password: &str) -> Self {
//         self.password = password.to_string();
//         self
//     }

//     pub fn with_user(mut self, user: &str) -> Self {
//         self.user = Some(user.to_string());
//         self
//     }

//     pub fn with_db(mut self, db: &str) -> Self {
//         self.db = Some(db.to_string());
//         self
//     }
// }

// impl IntoContainer for Postgres {
//     fn into_container(self) -> Container {
//         let mut container = PostgresBase::default()
//             .into_container()
//             .health_check(HealthCheck::new("pg_isready"))
//             .additional_wait_period(Duration::from_secs(2))
//             .wait_for(WaitStrategy::HealthCheck)
//             .env_var(("POSTGRES_PASSWORD", self.password));

//         if let Some(db) = self.db {
//             container = container.env_var(("POSTGRES_DB", db));
//         }

//         if let Some(user) = self.user {
//             container = container.env_var(("POSTGRES_USER", user));
//         }

//         container
//     }
// }
