use contain_rs::{
    client::{Client, Handle, Podman},
    container::IntoContainer,
};
use contain_rs_builder::{declare, env_var, health_check, image, port, wait_for_healthcheck};

#[test]
fn test_create_container() {
    let client = Podman::new();

    let container = declare(
        image("docker.io/library/nginx"),
        [
            port(8081, 80),
            env_var("SOME_VAR", "1"),
            health_check("curl http://localhost || exit 1", None, None, None, None),
            wait_for_healthcheck(),
        ],
    )
    .into_container();

    let handle = client.create(container);

    handle.run_and_wait();
    handle.rm();
}