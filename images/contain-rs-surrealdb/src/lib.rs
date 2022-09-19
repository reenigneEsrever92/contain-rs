use contain_rs_macro::ContainerImpl;

use contain_rs::container::{Container, Image, IntoContainer};

#[derive(ContainerImpl)]
#[container(image = "surrealdb/surrealdb:latest")]
struct SurrealDB;

#[cfg(test)]
mod test {
    #[test]
    fn test_connect() {}
}
