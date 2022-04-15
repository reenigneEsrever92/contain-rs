use contain_rs::{
    client::{podman::Podman, Client, Handle},
    container::{Container, HealthCheck, Image, WaitStrategy, postgres::Postgres},
};
use rstest::*;

use std::{str::FromStr, time::Duration};

#[fixture]
fn podman() -> Podman {
    Podman::new()
}

#[rstest]
#[case::podman_port_exposure(podman())]
// #[case::docker_port_exposure(docker(), "8082")]
fn test_wait_for_log(#[case] client: impl Client) {
    let container = Container::from_image(Image::from_name("docker.io/library/nginx")).wait_for(
        WaitStrategy::LogMessage {
            pattern: regex::Regex::from_str("ready for start up").unwrap(),
        },
    );

    client.run(&container).unwrap();

    std::thread::sleep(Duration::from_secs(1));

    assert!(client.wait(&container).is_ok());
}

#[rstest]
#[case::podman_port_exposure(podman())]
// #[case::docker_port_exposure(docker(), "8082")]
fn test_wait_for_health_check(#[case] client: impl Client) {
    let container = Container::from_image(Image::from_name("docker.io/library/nginx"))
        .health_check(HealthCheck::new("curl http://localhost || exit 1"))
        .wait_for(WaitStrategy::HealthCheck);

    client.run(&container).unwrap();

    assert!(client.wait(&container).is_ok());
}
