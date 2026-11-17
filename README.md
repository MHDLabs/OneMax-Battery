# OneMax Battery

[![CI](https://github.com/MHDLabs/OneMax-Battery/actions/workflows/ci.yml/badge.svg)](https://github.com/MHDLabs/OneMax-Battery/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/MHDLabs/OneMax-Battery?display_name=tag)](https://github.com/MHDLabs/OneMax-Battery/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A lightweight Rust userspace HID battery monitor for supported 2.4 GHz wireless mouse receivers on Linux.

> **Current hardware target:** ONE MAX OM-W45B — USB `1EA7:0066`.

## ✨ Features

- 🔎 **Automatic receiver detection** — finds the mouse HID interface by USB identity and its vendor report descriptor; it does not guess between the mouse and keyboard interfaces.
- 🔋 **Battery status** — reads the reported percentage using the tested vendor HID request (`0xB5 / 0x01`).
- 🖥️ **Waybar integration** — live JSON output, severity classes, and readable tooltips.
- 🔐 **User-session access** — an optional udev rule grants access to the active desktop user through `uaccess`; no permanent root daemon is needed.
- 🪶 **Small and focused** — Rust userspace program, no kernel module and no heavy runtime dependencies.
- 🧪 **Tested protocol** — the first response was verified on a real ONE MAX OM-W45B receiver.

## 📦 Install

Download the Linux x86_64 binary from [GitHub Releases](https://github.com/MHDLabs/OneMax-Battery/releases). The current public `v0.1.0` release contains the first manual-device build; automatic detection and Waybar monitoring are part of the upcoming `v0.2.0` development line until that tag is published.

Install the binary:

```sh
install -Dm755 one-max-battery-linux-x86_64 "$HOME/.local/bin/one-max-battery"
```

Make sure `$HOME/.local/bin` is in your `PATH`.

### Allow access without sudo

Install the udev rule:

```sh
sudo install -Dm644 contrib/udev/70-one-max-battery.rules \
  /etc/udev/rules.d/70-one-max-battery.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw
```

Unplug and reconnect the USB receiver. The rule uses systemd-logind's `uaccess` tag to grant the active desktop session access to this USB vendor/product pair. It does not grant broad access to every HID device.

## 🚀 Usage

Automatic detection is the default:

```sh
one-max-battery
```

The old explicit path form remains supported:

```sh
one-max-battery /dev/hidraw3
one-max-battery --device /dev/hidraw3
```

Continuous terminal monitoring:

```sh
one-max-battery --watch --interval 60
```

Print one Waybar JSON object:

```sh
one-max-battery --waybar
```

Show all options:

```sh
one-max-battery --help
```

## 🖥️ Waybar

The Waybar module runs the monitor as a continuous custom module and receives one JSON update per polling cycle. The default polling interval is 60 seconds.

1. Add `custom/onemax-battery` to your Waybar `modules-right` array.
2. Copy the object from [contrib/waybar/onemax-battery.jsonc](contrib/waybar/onemax-battery.jsonc) into your Waybar config.
3. Append [contrib/waybar/style.css](contrib/waybar/style.css) to your Waybar stylesheet.
4. Reload Waybar.

Waybar starts the monitor automatically when the bar starts and restarts it if it exits. When Sway already starts Waybar, no separate Sway `exec` line or system service is required. The module is managed by Waybar itself, avoiding a second background service.

The module displays:
- **Normal** — battery above 30%.
- **Warning** — battery from 16% to 30%.
- **Critical** — battery at 15% or below.
- **Disconnected / Error** — receiver missing, inaccessible, or not responding.

The example uses the Font Awesome battery icon (``). Use an icon font that provides this glyph, or replace it with an icon from your preferred Nerd Font.

## 🔬 Hardware protocol

The tested receiver reported:

```text
USB ID: 1EA7:0066
TX:     B5 01 00 00 00 00 00 00
RX:     B5 01 01 38 2E FF 09 70
```

The observed battery field was `0x38` (56 decimal). The status byte is shown as raw protocol data; its meaning is not yet documented and is not labelled as charging/full.

Automatic detection checks both the USB HID identity and the vendor-defined report descriptor, which distinguishes the mouse interface from the receiver's keyboard interface.

## 🛠️ Build and test

Requires Rust 1.74 or newer:

```sh
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

GitHub Actions runs formatting, tests, Clippy, and a Linux x86_64 release build. Pushing a version tag (for example, `v0.2.0`) publishes the executable and `SHA256SUMS`.

## ⚠️ Scope

- Only ONE MAX OM-W45B with USB ID `1EA7:0066` has been hardware-tested so far.
- Battery percentage is based on the device's reported byte; additional tests at multiple charge levels are welcome.
- Other receivers may use unrelated vendor protocols and are not assumed to be compatible.
- This is a userspace HID utility, not a kernel driver.

## 📄 License

MIT — see [LICENSE](LICENSE).
