use contain_rs::{
    client::{podman::Podman, Client, ContainerHandle},
    container::Container,
    image::Image,
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
    let mut container = Container::from_image(Image::from_name("docker.io/library/nginx"));

    container.map_port(port, 80);

    let mut handle = client.create(container);

    handle.run().unwrap();

    let response = reqwest::blocking::get(format!("http://localhost:{}", port)).unwrap();

    assert!(response.status().is_success());
    assert!(handle.stop().is_ok());
}

#[rstest]
#[case::podman_port_exposure(podman())]
// #[case::docker_port_exposure(docker(), "8082")]
fn test_port_exposure(#[case] client: impl Client) {
    let mut container = Container::from_image(Image::from_name("docker.io/library/nginx"));

    container.expose_port(80);

    let mut handle = client.create(container);

    handle.run().unwrap();

    let response = reqwest::blocking::get(format!(
        "http://localhost:{}",
        handle.get_exposed_port(80).unwrap().number
    ))
    .unwrap();

    assert!(response.status().is_success());
    assert!(handle.stop().is_ok());
}
