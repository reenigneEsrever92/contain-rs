use contain_rs::{
    client::{podman::Podman, Client, Handle},
    container::{Container, HealthCheck, Image, WaitStrategy},
};
use rstest::*;

#[fixture]
fn podman() -> Podman {
    Podman::new()
}

// #[fixture]
// fn docker() -> Docker {
//     Docker::new()
// }

#[rstest]
#[case::podman_port_exposure(podman(), 8081)]
// #[case::docker_port_exposure(docker(), "8082")]
fn test_map_exposure(#[case] client: impl Client, #[case] port: i32) {
    let container = Container::from_image(Image::from_name("docker.io/library/nginx"))
        .map_port(port, 80)
        .health_check(HealthCheck::new("curl http://localhost || exit 1"))
        .wait_for(WaitStrategy::HealthCheck);

    let handle = client.create(container);

    handle.run();

    let response = reqwest::blocking::get(format!("http://localhost:{}", port)).unwrap();

    assert!(response.status().is_success());

    handle.stop();
}
