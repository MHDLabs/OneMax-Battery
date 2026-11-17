mod linux;
mod protocol;

use std::{env, error::Error, process};

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let device = match args.next() {
        Some(arg) if arg == "--help" || arg == "-h" => {
            println!("OneMax Battery 0.1.0\n\nUsage: one-max-battery /dev/hidrawX");
            return Ok(());
        }
        Some(device) => device,
        None => {
            return Err("Usage: one-max-battery /dev/hidrawX".into());
        }
    };

    if args.next().is_some() {
        return Err("Unexpected extra argument. Usage: one-max-battery /dev/hidrawX".into());
    }

    println!("OneMax Battery v{}", env!("CARGO_PKG_VERSION"));
    println!("Device: {device}");

    let battery = linux::read_battery(&device)?;
    println!("RX: {}", protocol::format_hex(&battery.raw));
    println!("Status: 0x{:02X}", battery.status);
    println!("Battery: {}%", battery.percent);

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
