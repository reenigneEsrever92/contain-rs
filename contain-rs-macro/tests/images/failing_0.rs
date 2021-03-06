use contain_rs::{
    client::{Client, Podman},
    container::{Container, IntoContainer},
};
use contain_rs_macro::Container;

#[derive(Default, Container)]
#[container(
    image = "docker.io/library/nginx",
    health_check_command = "curl http://localhost || exit 1",
    health_check_timeout = 30000
)]
#[container()]
struct SimpleImage {
    #[env_var("PG_PASSWORD")]
    password: String,
}

fn main() {
    let podman = Podman::new();
    // let container = SimpleImage::default().into_container();

    // podman.run(&container).unwrap();
    // podman.wait(&container).unwrap();
    // podman.rm(&container).unwrap();
}
