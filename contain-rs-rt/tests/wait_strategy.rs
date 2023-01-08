use contain_rs_rt::{
    client::{docker::Docker, podman::Podman, Client},
    container::{Container, HealthCheck, Image, WaitStrategy},
};
use rstest::*;

use std::{str::FromStr, time::Duration};

#[fixture]
fn podman() -> Podman {
    Podman::new()
}

#[fixture]
fn docker() -> Docker {
    Docker::new()
}

#[rstest]
#[case::podman_wait_for_log(podman())]
#[case::docker_wait_for_log(docker())]
fn test_wait_for_log(#[case] client: impl Client) {
    let mut container = Container::from_image(Image::from_str("docker.io/library/nginx").unwrap());

    container.wait_for(WaitStrategy::LogMessage {
        pattern: regex::Regex::from_str("ready for start up").unwrap(),
    });

    client.run(&container).unwrap();

    std::thread::sleep(Duration::from_secs(2));

    client.wait(&container).unwrap();
    client.rm(&container).unwrap();
}

#[rstest]
#[case::podman_wait_for_healthcheck(podman())]
#[case::docker_wait_for_healthcheck(docker())]
fn test_wait_for_health_check(#[case] client: impl Client) {
    let mut container = Container::from_image(Image::from_str("docker.io/library/nginx").unwrap());

    container
        .health_check(HealthCheck::new("curl http://localhost || exit 1"))
        .wait_for(WaitStrategy::HealthCheck);

    client.run(&container).unwrap();
    client.wait(&container).unwrap();
    client.rm(&container).unwrap();
}
