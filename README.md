![Test](https://github.com/reenigneEsrever92/contain-rs/actions/workflows/rust.yml/badge.svg)

# contain-rs

A tool to use docker and podman containers with rust.

For usage take a look at the [Documentation](https://docs.rs/contain-rs/0.1.3/contain_rs/)

## Quick Start

Add containers to your Cargo.toml

```toml
[dependencies]
contain-rs = "0.2"
```

Create a client and start a container:

```rust
use contain_rs::{
    client::{docker::Docker, Client, Handle},
    container::{Container, Image},
};

let docker = Docker::new();

let container = Container::from_image(Image::from_name("docker.io/library/nginx"));

let handle = podman.create(container);

handle.run();
handle.wait();
handle.rm();
```
