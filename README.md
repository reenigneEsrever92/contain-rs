![example workflow](https://github.com/reenigneEsrever92/contain-rs/actions/workflows/rust.yml/badge.svg)

# contain-rs
A tool to use podman containers with rust.

Docker is planned, as well. I just happened to start with podman since I tend to like a lot.

## TODO

- [x] improve error types
- [x] improve error reporting
- [x] handle std error for child processes
- [x] implement exposed ports
- [x] check status before running commands
- [x] extract comman parts into shared module
- [x] implement healthcheck wait strategy
- [ ] create image macro, to create new images from structs easily
- [ ] care for stderr
- [ ] add docker implementation
- [ ] add a few images

