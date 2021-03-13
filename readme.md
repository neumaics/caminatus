# Caminatus
Raspberry Pi Kiln Controller

<del>Ripped off from</del> Inspired by https://github.com/jbruce12000/kiln-controller

**Warning**: this system not yet been tested on a real kiln/oven/furnace. If you somehow manage
to get it working in its current state, you use at your own risk! Not intended for exposure to the
public internet, for reasons that I hope are obvious.

## Purpose
Caminatus can be used to control kilns and furnaces. It was originally made to test different
control theory algorithms, but now only, reliably, incorporates PID control. Eventually fuzzy
logic, and other algorithms may be incorporated.


## Setup
See setup for jbruce12000/kiln-controller for a similar direction for Caminatus.

## Running
```bash
> ./caminatus --config-file=./path/to/config.yaml \
              --schedules-folder=./path/to/schedules

```

Where

`--config-file` - a path to a config yaml. An example can be found at 
`./config.yaml.example`

`--schedules-folder` - a path to the folder where schedules are managed

## Development
Requirements:
* Rust 1.47.0+
* Nodejs v14+

This was developed primarily with the vs-code remote container and docker.
The ./.devcontainer/dockerfile was modified from the [rust devcontainer](https://github.com/microsoft/vscode-remote-try-rust). It includes node/npm,
rust/cargo, fish and other utilties to aid in making and testing changes.

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
