use contain_rs::{
    client::{Client, Podman},
    container::{Container, IntoContainer},
};
use contain_rs_builder::{declare, image};
use contain_rs_macro::container;

#[container(declare(image("docker.io/library/nginx"), []))]
struct SimpleImage;

impl SimpleImage {
    fn new() -> Self {
        Self {}
    }
}

fn main() {
    let podman = Podman::new();
    let container = SimpleImage::new().into_container();

    podman.run(&container).unwrap();
    podman.wait(&container).unwrap();
    podman.rm(&container).unwrap();
}
