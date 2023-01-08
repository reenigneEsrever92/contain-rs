use contain_rs_macro::ContainerImpl;
use contain_rs_rt::container::{Container, HealthCheck, Image, IntoContainer, WaitStrategy};

#[derive(ContainerImpl, Default)]
#[container(image = "docker.io/library/nginx", health_check_command = "curl http://localhost || exit 1", ports = [8080:80])]
struct Nginx;

#[cfg(test)]
mod test {
    use contain_rs_rt::client::{podman::Podman, Client, Handle};

    use crate::Nginx;

    #[test]
    fn test_get() {
        let client = Podman::default();

        let container = client.create(Nginx::default());

        container.run().unwrap();
        container.wait().unwrap();

        let request = reqwest::blocking::get("http://localhost:8080").unwrap();

        assert!(request.status().is_success());
    }
}
