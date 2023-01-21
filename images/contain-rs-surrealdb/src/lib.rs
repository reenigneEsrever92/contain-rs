use contain_rs::*;

#[derive(Default, ContainerImpl)]
#[container(
    image = "docker.io/surrealdb/surrealdb:latest", 
    command = ["start"], 
    ports = [8080:8000], 
    wait_log = "Started web server on"
)]
pub struct SurrealDB {
    #[arg = "--user"]
    user: Option<String>,
    #[arg = "--pass"]
    password: Option<String>,
}

#[cfg(test)]
mod test {
    use contain_rs::{Client, Docker, Handle};
    use tracing_subscriber::filter::LevelFilter;

    use crate::SurrealDB;

    #[test]
    fn test_surrealdb() {
        tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE).init();

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
