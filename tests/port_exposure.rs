use contain_rs::{
    client::{podman::Podman, Client, Handle},
    container::{Container, Image},
};
use rstest::*;

use std::str::FromStr;

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
    pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .init();

    let mut container = Container::from_image(Image::from_name("docker.io/library/nginx"))
        .map_port(port, 80)
        .wait_for(contain_rs::container::WaitStrategy::LogMessage {
            pattern: regex::Regex::from_str("ready for start up").unwrap(),
        });

    let mut handle = client.create(container);

    handle.run();

    let response = reqwest::blocking::get(format!("http://localhost:{}", port)).unwrap();

    assert!(response.status().is_success());

    handle.stop()
}
