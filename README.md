# OneMax Battery

A lightweight Rust userspace HID tool for reading battery status from supported 2.4 GHz wireless mouse receivers on Linux.

> **Status:** Early development. The first target is the ONE MAX OM-W45B receiver (`1EA7:0066`).

## Project goals

- Read battery status through Linux `hidraw`.
- Keep device communication in userspace; no kernel module.
- Validate the target USB receiver before sending a protocol request.
- Provide small, downloadable Linux binaries through GitHub Releases.
- Keep the implementation minimal and auditable.

## Hardware discovery

The target receiver exposes a vendor-defined HID report (Report ID `0xB5`). A one-shot request was tested successfully on the developer's device:

```text
TX: B5 01 00 00 00 00 00 00
RX: B5 01 01 38 2E FF 09 70
```

The observed battery field was `0x38` (56 decimal). The request/response mapping is an early protocol finding and still needs validation across reconnects, battery levels, and additional devices.

## Build

Requires Rust and Cargo:

```sh
cargo build --release
```

## Run

The program will require read/write access to the matching `hidraw` node:

```sh
sudo ./target/release/one-max-battery /dev/hidrawX
```

Use the correct node for the mouse interface, not the keyboard interface.

## Safety

The initial implementation is intended to send one documented battery-status query only. It must not send undocumented commands or change DPI, lighting, pairing, or other device settings.

## Documentation

- [فارسی](README-fa.md)

## License

GPL-3.0-only.
