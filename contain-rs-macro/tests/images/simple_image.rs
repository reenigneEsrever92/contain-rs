use contain_rs::*;

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
