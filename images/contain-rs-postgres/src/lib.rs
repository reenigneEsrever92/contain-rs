use contain_rs::*;

#[derive(ContainerImpl)]
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

impl Default for Postgres {
    fn default() -> Self {
        Self {
            db: None,
            user: None,
            password: "default_pw".to_string(),
        }
    }
}

impl Postgres {
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

#[cfg(test)]
mod test {
    use contain_rs::{Client, Handle, Podman};

    use crate::Postgres;

    #[test]
    fn test_run() {
        let client = Podman::new();
        let container = client.create(Postgres::default());

        container.run_and_wait().unwrap();
    }
}
