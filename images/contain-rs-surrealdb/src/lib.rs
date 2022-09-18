use contain_rs_macro::Container;

use contain_rs::container::{Container, Image, IntoContainer};

#[derive(Container)]
#[container(image = "surrealdb/surrealdb:latest")]
struct SurrealDB;

#[cfg(test)]
mod test {
    #[test]
    fn test_connect() {}
}
