use contain_rs_macro::ContainerImpl;

use contain_rs::container::{Container, Image, IntoContainer};

#[derive(ContainerImpl)]
#[container(image = "docker.io/surrealdb/surrealdb:latest", command = ["start"])]
struct SurrealDB;

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use contain_rs::client::{Client, Docker, Handle};

    use crate::SurrealDB;

    #[test]
    fn test_connect() {
        let client = Docker::new();
        let container = client.create(SurrealDB);

        container.run().unwrap();

        thread::sleep(Duration::from_secs(20))
    }
}
