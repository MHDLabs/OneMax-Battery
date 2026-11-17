# OneMax Battery

A small Rust userspace HID utility for reading battery status from supported 2.4 GHz wireless mouse receivers on Linux.

**Status:** Early development. The first target is the ONE MAX OM-W45B receiver, USB ID **1EA7:0066**.

## What it does

- Opens the matching Linux hidraw interface.
- Verifies the USB HID ID before sending anything.
- Sends one battery-status request (Report ID B5, command 01).
- Waits up to 750 ms for a matching response.
- Prints the reported battery percentage.
- Does not load a kernel module or change mouse settings.

## Hardware protocol observation

A request was tested on the target receiver:

```text
TX: B5 01 00 00 00 00 00 00
RX: B5 01 01 38 2E FF 09 70
```

The observed battery field was **0x38 (56 decimal)**. The response layout is currently based on this successful device test and a related protocol reference. More testing across battery levels and reconnects is needed before treating the protocol as universal.

## Build from source

Requires Rust and Cargo:

```sh
cargo test
cargo build --release
```

## Run

Find the mouse interface (not the keyboard interface):

```sh
ls -l /dev/hidraw*
```

Then run:

```sh
sudo ./target/release/one-max-battery /dev/hidrawX
```

The process needs read/write access to the matching hidraw node. The program checks the sysfs HID ID and refuses to send the request to a different USB device.

## Releases

Pushing a version tag (for example, `v0.1.0`) triggers a GitHub Actions build for **Linux x86_64**. The release contains:

- `one-max-battery-linux-x86_64`
- `SHA256SUMS`

The release binary is built on Ubuntu 22.04 to keep its glibc requirement broadly compatible with current Linux distributions.

## Development status

This is an experimental userspace driver, not a kernel driver. At this stage it supports only the tested receiver ID and one battery query. Automatic device selection, periodic monitoring, desktop integration, and other receiver models are not implemented.

## Documentation

- [Persian documentation](README-fa.md)

## License

MIT. See [LICENSE](LICENSE).
