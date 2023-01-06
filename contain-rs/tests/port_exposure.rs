use contain_rs::{
    client::{podman::Podman, Client, Docker, Handle},
    container::{Container, HealthCheck, Image, WaitStrategy},
};
use rstest::*;

use std::str::FromStr;

#[fixture]
fn podman() -> Podman {
    Podman::new()
}

#[fixture]
fn docker() -> Docker {
    Docker::new()
}

#[rstest]
#[case::podman_port_exposure(podman(), 8081)]
#[case::docker_port_exposure(docker(), 8082)]
fn test_map_exposure(#[case] client: impl Client, #[case] port: u32) {
    let mut container = Container::from_image(Image::from_str("docker.io/library/nginx").unwrap());

    container
        .map_port(port, 80)
        .health_check(HealthCheck::new("curl http://localhost || exit 1"))
        .wait_for(WaitStrategy::HealthCheck);

    let handle = client.create(container);

    handle.run().unwrap();
    handle.wait().unwrap();

    let response = reqwest::blocking::get(format!("http://localhost:{}", port)).unwrap();

    assert!(response.status().is_success());

    handle.stop().unwrap();
}
