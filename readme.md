# Caminatus
Raspberry Pi Kiln Controller

<del>Ripped off from</del> Inspired by https://github.com/jbruce12000/kiln-controller

**Warning**: this system not yet been tested on a real kiln/oven/furnace. If you somehow manage
to get it working in its current state, you use at your own risk! Not intended for exposure to the
public internet.

## Purpose
Caminatus can be used to control kilns and furnaces.

## Setup
TBD

## Running
```bash
> ./caminatus --config-file=./path/to/config.yaml \
              --schedules-folder=./path/to/schedules

```

## Development
Requirements:
* Rust 1.47.0+
* Nodejs v14+

This was developed primarily with the vs-code remote container and docker.
The ./.devcontainer/dockerfile was modified from the rust devcontainer. It includes node/npm,
rust/cargo, fish and other utilties.

### Packaging for the Raspberry Pi
Requirements:
* Docker

Caminatus has only been tested on Raspberry Pi Zero and 3, but should be able to work on the other
Raspberry Pi models. Builds for Raspberry Pis currently rely on
[cargo-make](https://github.com/sagiegurari/cargo-make).
Tasks for building are defined in [makefile.toml]('./makefile.toml).

To package changes for Raspberry Pi 1 or Zero:
```bash
> cargo make cross-package --profile rpi-0
```

To package changes for Raspberry Pi 2, 3, or 4:
```bash
> cargo make cross-package --profile rpi-4
```

The resulting archive can be found in `./target/caminautus-(name)-(version).tar.gz` which can be
copied to your device of choice.
