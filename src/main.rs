mod linux;
mod protocol;

use std::{
    env,
    error::Error,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
    thread,
    time::Duration,
};

use linux::read_battery;
use protocol::{battery_class, format_hex, BatteryResponse};

const DEFAULT_INTERVAL: u64 = 60;
const MIN_INTERVAL: u64 = 5;
const MAX_INTERVAL: u64 = 3600;

#[derive(Debug)]
struct Options {
    device: Option<PathBuf>,
    watch: bool,
    waybar: bool,
    interval: u64,
}

fn print_help() {
    println!(
        "OneMax Battery v{}

A lightweight userspace HID battery monitor for Linux.

USAGE:
    one-max-battery [OPTIONS] [DEVICE]

OPTIONS:
    -d, --device <PATH>    Use a specific hidraw device (for example /dev/hidraw3)
        --auto             Automatically detect the OneMax receiver (default)
    -w, --watch            Keep monitoring and print updates continuously
        --waybar           Emit Waybar-compatible JSON instead of human-readable text
    -i, --interval <SEC>   Poll interval in seconds (5..=3600, default: {})
    -h, --help             Print this help
    -V, --version          Print version

EXAMPLES:
    one-max-battery
    one-max-battery --device /dev/hidraw3
    one-max-battery --watch --interval 60
    one-max-battery --watch --waybar --interval 60

WAYBAR:
    Add the custom/onemax-battery module from the repository's
    contrib/waybar/onemax-battery.jsonc example.

The monitor sends only the tested battery-status query. It does not change
DPI, lighting, pairing, or other mouse settings.",
        env!("CARGO_PKG_VERSION"),
        DEFAULT_INTERVAL
    );
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn parse_options() -> Result<Option<Options>, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut device = None;
    let mut watch = false;
    let mut waybar = false;
    let mut interval = DEFAULT_INTERVAL;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("one-max-battery {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "-d" | "--device" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("--device requires a path"))?;
                if device.replace(PathBuf::from(value)).is_some() {
                    return Err(invalid_input("device path was specified more than once").into());
                }
            }
            "--auto" => {
                if device.is_some() {
                    return Err(invalid_input("--auto cannot be combined with --device").into());
                }
            }
            "-w" | "--watch" => watch = true,
            "--waybar" => waybar = true,
            "-i" | "--interval" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid_input("--interval requires a number of seconds"))?;
                interval = value.parse::<u64>().map_err(|_| {
                    invalid_input(format!("invalid interval: {value}"))
                })?;
                if !(MIN_INTERVAL..=MAX_INTERVAL).contains(&interval) {
                    return Err(invalid_input(format!(
                        "--interval must be between {MIN_INTERVAL} and {MAX_INTERVAL} seconds"
                    ))
                    .into());
                }
            }
            value if value.starts_with('-') => {
                return Err(invalid_input(format!("unknown option: {value}")).into());
            }
            value => {
                if device.replace(PathBuf::from(value)).is_some() {
                    return Err(invalid_input(
                        "only one device path may be specified",
                    )
                    .into());
                }
            }
        }
        index += 1;
    }

    Ok(Some(Options {
        device,
        watch,
        waybar,
        interval,
    }))
}

fn query(device_override: Option<&Path>) -> io::Result<(PathBuf, BatteryResponse)> {
    let device = match device_override {
        Some(path) => path.to_path_buf(),
        None => linux::find_device()?,
    };

    let response = read_battery(&device.to_string_lossy())?;
    Ok((device, response))
}

fn print_human_result(result: &io::Result<(PathBuf, BatteryResponse)>) {
    match result {
        Ok((device, battery)) => {
            println!("OneMax Battery v{}", env!("CARGO_PKG_VERSION"));
            println!("Device: {}", device.display());
            println!("RX: {}", format_hex(&battery.raw));
            println!("Status: 0x{:02X}", battery.status);
            println!("Battery: {}%", battery.percent);
        }
        Err(error) => {
            eprintln!("OneMax Battery: {error}");
        }
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }

    escaped
}

fn emit_waybar_result(result: &io::Result<(PathBuf, BatteryResponse)>) {
    match result {
        Ok((device, battery)) => {
            let class = battery_class(battery.percent);
            let tooltip = format!(
                "ONE MAX OM-W45B\nBattery: {}%\nUSB: 1EA7:0066\nInterface: {}\nReport status: 0x{:02X}",
                battery.percent,
                device.display(),
                battery.status
            );
            println!(
                "{{\"text\":\"{}%\",\"tooltip\":\"{}\",\"class\":\"{}\",\"percentage\":{}}}",
                battery.percent,
                json_escape(&tooltip),
                class,
                battery.percent
            );
        }
        Err(error) => {
            let (text, class, tooltip) = match error.kind() {
                io::ErrorKind::NotFound => (
                    "—",
                    "disconnected",
                    "ONE MAX receiver not found\nConnect the 2.4 GHz USB receiver.",
                ),
                io::ErrorKind::PermissionDenied => (
                    "LOCK",
                    "error",
                    "Cannot access the mouse receiver.\nInstall the udev rule from contrib/udev and reconnect the receiver.",
                ),
                io::ErrorKind::TimedOut => (
                    "—",
                    "error",
                    "The receiver did not answer the battery query.\nCheck the receiver connection and try again.",
                ),
                _ => ("ERR", "error", "OneMax Battery could not read the receiver."),
            };

            let detail = format!("{tooltip}\n{error}");
            println!(
                "{{\"text\":\"{}\",\"tooltip\":\"{}\",\"class\":\"{}\"}}",
                text,
                json_escape(&detail),
                class
            );
        }
    }

    let _ = io::stdout().flush();
}

fn run(options: Options) -> Result<(), Box<dyn Error>> {
    if options.watch {
        loop {
            let result = query(options.device.as_deref());

            if options.waybar {
                emit_waybar_result(&result);
            } else {
                print_human_result(&result);
                println!();
            }

            thread::sleep(Duration::from_secs(options.interval));
        }
    }

    let result = query(options.device.as_deref());

    if options.waybar {
        emit_waybar_result(&result);
        return Ok(());
    }

    let (device, battery) = result?;
    print_human_result(&Ok((device, battery)));
    Ok(())
}

fn main() {
    let result = parse_options().and_then(|options| match options {
        Some(options) => run(options),
        None => Ok(()),
    });

    if let Err(error) = result {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
