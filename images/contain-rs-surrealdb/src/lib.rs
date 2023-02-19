use contain_rs::*;

#[derive(Clone, ContainerImpl)]
#[container(
    image = "docker.io/surrealdb/surrealdb:latest", 
    command = ["start"], 
    wait_log = ".*Started web server on.*"
)]
pub struct SurrealDB {
    #[contain_rs(arg = "--user")]
    user: Option<String>,
    #[contain_rs(arg = "--pass")]
    password: Option<String>,
    #[contain_rs(port = 8000)]
    port: u32,
}

impl Default for SurrealDB {
    fn default() -> Self {
        Self {
            user: Default::default(),
            password: Default::default(),
            port: 8080,
        }
    }
}

impl SurrealDB {
    #[allow(unused)]
    pub fn user(&mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self.clone()
    }

    #[allow(unused)]
    pub fn password(&mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self.clone()
    }

    pub fn port(&mut self, port: u32) -> Self {
        self.port = port;
        self.clone()
    }
}

#[cfg(test)]
mod test {
    use contain_rs::{Client, Docker, Handle};

    use crate::SurrealDB;

    #[test]
    fn test_surrealdb() {
        let client = Docker::new();
        let container = client.create(SurrealDB::default());

        container.run_and_wait().unwrap();

        let client = reqwest::blocking::Client::new();

        let request = client
            .post("http://localhost:8080/sql")
            .header("Accept", "application/json")
            .header("NS", "test")
            .header("DB", "test")
            .body("create type::table(\"test_table\")")
            .build()
            .unwrap();

        let response = client.execute(request).unwrap();

        assert_eq!(reqwest::StatusCode::OK, response.status())
    }
}
