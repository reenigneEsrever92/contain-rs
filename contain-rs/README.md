# contain-rs

A tool to use docker and podman containers with rust.

For usage take a look at the [Documentation](https://docs.rs/contain-rs/0.1.3/contain_rs/)

## Basic usage

```
use contain_rs::{Podman, Client, Handle, Container, Image};
use std::str::FromStr;

let podman = Podman::new();

let container = Container::from_image(Image::from_str("docker.io/library/nginx").unwrap());

let handle = podman.create(container);

handle.run();
handle.wait();

// when the handle gets out of scope the container is stopped and removed
```

## Clients

Clients are used for scheduling containers. There are currently two implementations available.
One of them works with docker the other one uses podman.

## Images

Containers need image to run. You can create images like so:

```
use contain_rs::Image;
use std::str::FromStr;

let image = Image::from_str("docker.io/library/nginx");

assert!(image.is_ok());

let latest = Image::from_str("docker.io/library/nginx:latest");

assert!(latest.is_ok());
```

## Macro

Contain-rs provides a derive macro to implement the IntoContainer trait for a struct. 
The macros feature has to be enabled to make use of the derive macro.

You can then create a container like this:

```rust
use contain_rs::*;

#[derive(ContainerImpl, Default)]
#[container(
    image = "docker.io/library/nginx",
    health_check_command = "curl http://localhost || exit 1"
)]
struct Nginx {
    #[contain_rs(port = 80)]
    port: u32,
}

let client = Docker::default();

let nginx = client.create(Nginx{ port: 8080 });

nginx.run();
nginx.wait();
nginx.rm(); // stops and removes the container
// this would be done automatically when the container handle goes out of scope
```


