use contain_rs::{
    client::{Client, Podman},
    container::{Container, Image, IntoContainer},
};
use contain_rs_macro::container;

#[container(image = "docker.io/library/nginx")]
struct SimpleImage;

fn main() {
    let podman = Podman::new();
    let container = SimpleImage::new().into_container();

    podman.run(&container).unwrap();
    podman.wait(&container).unwrap();
    podman.rm(&container).unwrap();
}
