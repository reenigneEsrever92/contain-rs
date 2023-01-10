use contain_rs_core::container::{Container, HealthCheck, Image, IntoContainer, WaitStrategy};
use contain_rs_macro::ContainerImpl;
use std::str::FromStr;

#[derive(Default, ContainerImpl)]
#[container(
    image = "docker.io/library/nginx",
    health_check_command = "curl http://localhost || exit 1",
    health_check_timeout = 30000
)]
struct SimpleImage {
    #[env_var("PG_PASSWORD")]
    password: String,
}

fn main() {
    // let podman = Podman::new();
    // let container = SimpleImage::default().into_container();

    // podman.run(&container).unwrap();
    // podman.wait(&container).unwrap();
    // podman.rm(&container).unwrap();
}
