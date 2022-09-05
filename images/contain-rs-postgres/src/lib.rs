///
/// This module provides postgres image utilities.
///
/// ## Usage:
///
///
use contain_rs_macro::Container;

#[derive(Container)]
#[container(
    image = "docker.io/library/postgres",
    health_check_command = "pg_isready"
)]
pub struct Postgres {
    #[env_var = "POSTGRES_DB"]
    db: Option<String>,
    #[env_var = "POSTGRES_USER"]
    user: Option<String>,
    #[env_var = "POSTGRES_PASSWORD"]
    password: String,
}

impl Postgres {
    pub fn default() -> Self {
        Self {
            db: None,
            user: None,
            password: "default_pw".to_string(),
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
}
