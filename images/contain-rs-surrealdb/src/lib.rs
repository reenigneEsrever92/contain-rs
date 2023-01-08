use contain_rs::*;

#[derive(ContainerImpl)]
#[container(image = "docker.io/surrealdb/surrealdb:latest", command = ["start"], wait_time = 2000 )]
struct SurrealDB;

#[cfg(test)]
mod test {
    use contain_rs::{Client, Docker, Handle};

    use crate::SurrealDB;

    #[test]
    fn test_connect() {
        let client = Docker::new();
        let container = client.create(SurrealDB);

        container.run().unwrap();
    }
}
