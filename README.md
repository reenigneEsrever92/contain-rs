![example workflow](https://github.com/reenigneEsrever92/contain-rs/actions/workflows/rust.yml/badge.svg)

# contain-rs

A tool to use podman containers with rust.

Docker is planned, as well. I just happened to start with podman since I like it a lot.

For usage take a look at the [Documentation](https://docs.rs/contain-rs/0.1.3/contain_rs/)

## TODO

- [x] improve error types
- [x] improve error reporting
- [x] handle std error for child processes
- [x] implement exposed ports
- [x] check status before running commands
- [x] extract comman parts into shared module
- [x] implement healthcheck wait strategy
- [ ] add docker implementation
- [ ] create image macro, to create new images from structs easily
- [ ] add env vars and flags to parameterize clis (DOCKER_HOST, etc.)
- [ ] add a few images

