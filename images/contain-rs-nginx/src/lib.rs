use contain_rs::container::{Container, Image, IntoContainer};
use contain_rs_macro::Container;

#[derive(Container)]
#[container(image = "docker.io/library/postgres")]
struct Nginx;
