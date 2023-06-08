# flux-rs-sys

Rust bindings (and examples) for Flux core.

## Development

A VSCode [Developer Environment](https://code.visualstudio.com/learn/develop-cloud/overview)
is included for your development. It provides an installation of Flux alongside the rust toolchain.
You can open the environment by opening VSCode -> View -> Command Palette -> Dev Containers: Rebuild Container.
Once in the container (or in an environment with cargo) you can do:

```bash
$ cargo update
$ cargo build
```

And run tests:

```bash
$ cargo test
```