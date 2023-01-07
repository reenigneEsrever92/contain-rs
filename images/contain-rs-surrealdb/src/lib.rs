use contain_rs_macro::ContainerImpl;

use contain_rs_rt::container::{Container, Image, IntoContainer, WaitStrategy};
use std::str::FromStr;
use std::time::Duration;

#[derive(ContainerImpl)]
#[container(image = "docker.io/surrealdb/surrealdb:latest", command = ["start"], wait_time = 2000 )]
struct SurrealDB;

#[cfg(test)]
mod test {
    use contain_rs_rt::client::{Client, Docker, Handle};

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
