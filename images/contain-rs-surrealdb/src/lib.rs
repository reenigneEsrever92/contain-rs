use contain_rs_macro::ContainerImpl;

use contain_rs::container::{Container, Image, IntoContainer};
use std::str::FromStr;

#[derive(ContainerImpl)]
#[container(image = "docker.io/surrealdb/surrealdb:latest", command = ["start"], wait_time = "2s" )]
struct SurrealDB;

#[cfg(test)]
mod test {
    use contain_rs::client::{Client, Docker, Handle};

    use crate::SurrealDB;

    #[test]
    fn test_connect() {
        // pretty_env_logger::formatted_timed_builder()
        //     .filter_level(log::LevelFilter::Trace)
        //     .init();

        let client = Docker::new();
        let container = client.create(SurrealDB);

        container.run().unwrap();
    }
}
