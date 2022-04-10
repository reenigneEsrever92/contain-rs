use contain_rs::{
    client::{podman::Podman, Client, ContainerHandle},
    container::Container,
    image::Image,
};

#[test]
pub fn test_port_exposure() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let podman = Podman::new();
    let mut container = Container::from_image(Image::from_name("docker.io/library/nginx"));

    container.expose_port("80", "8081");

    let mut handle = podman.create(container).unwrap();

    assert!(handle.run().is_ok());
    assert!(handle.stop().is_ok());
}
