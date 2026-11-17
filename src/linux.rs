use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::fd::AsRawFd,
    path::Path,
    time::{Duration, Instant},
};

use crate::protocol::{parse_response, BatteryResponse, ParseResult, BATTERY_REQUEST};

const USB_VID: &str = "1EA7";
const USB_PID: &str = "0066";
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(750);

fn verify_target_hidraw(device: &str) -> io::Result<()> {
    let path = Path::new(device);
    let node = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.starts_with("hidraw"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "device path must point to a /dev/hidrawX node",
            )
        })?;

    let uevent_path = format!("/sys/class/hidraw/{node}/device/uevent");
    let uevent = fs::read_to_string(&uevent_path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("cannot inspect {uevent_path}: {error}"),
        )
    })?;

    let hid_id = uevent
        .lines()
        .find_map(|line| line.strip_prefix("HID_ID="))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "HID_ID is missing from the hidraw sysfs uevent",
            )
        })?;

    let fields: Vec<_> = hid_id.split(':').collect();
    if fields.len() != 3
        || !fields[1].eq_ignore_ascii_case(USB_VID)
        || !fields[2].eq_ignore_ascii_case(USB_PID)
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("unsupported receiver (HID_ID={hid_id}); expected USB {USB_VID}:{USB_PID}"),
        ));
    }

    Ok(())
}

fn wait_readable(file: &File, timeout: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }

        let timeout_ms = remaining.as_nanos().div_ceil(1_000_000)
            .max(1)
            .min(i32::MAX as u128) as i32;

        let mut descriptor = libc::pollfd {
            fd: file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };

        let result = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };

        if result > 0 {
            if descriptor.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                return Err(io::Error::other("hidraw reported an I/O error"));
            }

            return Ok(descriptor.revents & libc::POLLIN != 0);
        }

        if result == 0 {
            return Ok(false);
        }

        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

pub fn read_battery(device: &str) -> io::Result<BatteryResponse> {
    verify_target_hidraw(device)?;

    let mut file = OpenOptions::new().read(true).write(true).open(device)?;

    // One documented battery-status request; no configuration commands.
    file.write_all(&BATTERY_REQUEST)?;
    file.flush()?;

    let deadline = Instant::now() + RESPONSE_TIMEOUT;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() || !wait_readable(&file, remaining)? {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "no battery response received within 750 ms",
            ));
        }

        let mut buffer = [0_u8; 64];
        let size = match file.read(&mut buffer) {
            Ok(size) => size,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };

        if size == 0 {
            continue;
        }

        match parse_response(&buffer[..size]) {
            ParseResult::Unrelated => continue,
            ParseResult::Invalid(message) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, message));
            }
            ParseResult::Battery(response) => return Ok(response),
        }
    }
}
