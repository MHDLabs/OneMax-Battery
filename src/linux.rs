use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::protocol::{parse_response, BatteryResponse, ParseResult, BATTERY_REQUEST};

const USB_BUS_TYPE: u32 = 0x0003;
const USB_VID: u32 = 0x1EA7;
const USB_PID: u32 = 0x0066;
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(750);
const HIDRAW_SYSFS: &str = "/sys/class/hidraw";

const BATTERY_REPORT_SIGNATURE: [u8; 9] = [
    0x06, 0xB5, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x85, 0xB5,
];

fn is_target_hid_id(hid_id: &str) -> bool {
    let mut fields = hid_id.split(':');
    let (Some(bus_type), Some(vendor), Some(product), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return false;
    };

    u32::from_str_radix(bus_type, 16).ok() == Some(USB_BUS_TYPE)
        && u32::from_str_radix(vendor, 16).ok() == Some(USB_VID)
        && u32::from_str_radix(product, 16).ok() == Some(USB_PID)
}

fn has_battery_report(descriptor: &[u8]) -> bool {
    descriptor
        .windows(BATTERY_REPORT_SIGNATURE.len())
        .any(|window| window == BATTERY_REPORT_SIGNATURE)
}

fn read_hid_metadata(node: &str) -> io::Result<(String, Vec<u8>)> {
    let device_path = Path::new(HIDRAW_SYSFS).join(node).join("device");
    let uevent_path = device_path.join("uevent");
    let uevent = fs::read_to_string(&uevent_path)?;

    let hid_id = uevent
        .lines()
        .find_map(|line| line.strip_prefix("HID_ID="))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("HID_ID is missing from {}", uevent_path.display()),
            )
        })?;

    let canonical_device = fs::canonicalize(&device_path).unwrap_or(device_path);
    let descriptor = fs::read(canonical_device.join("report_descriptor"))?;

    Ok((hid_id.to_owned(), descriptor))
}

fn verify_target_hidraw(device: &Path) -> io::Result<()> {
    let node = device
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.starts_with("hidraw"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "device path must point to a /dev/hidrawX node",
            )
        })?;

    let (hid_id, descriptor) = read_hid_metadata(node)?;

    if !is_target_hid_id(&hid_id) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("unsupported receiver (HID_ID={hid_id}); expected USB 1EA7:0066"),
        ));
    }

    if !has_battery_report(&descriptor) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{} belongs to the receiver but does not expose the expected battery report",
                device.display()
            ),
        ));
    }

    Ok(())
}

pub fn find_device() -> io::Result<PathBuf> {
    let entries = fs::read_dir(HIDRAW_SYSFS)?;
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let node = entry.file_name();
        let Some(node_name) = node.to_str() else {
            continue;
        };

        if !node_name.starts_with("hidraw") {
            continue;
        }

        let device = Path::new("/dev").join(node_name);
        if verify_target_hidraw(&device).is_ok() {
            candidates.push(device);
        }
    }

    candidates.sort();

    match candidates.as_slice() {
        [device] => Ok(device.clone()),
        [] => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no supported ONE MAX receiver interface found (USB 1EA7:0066, HID report 0xB5)",
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "multiple supported mouse interfaces found: {}; select one with --device",
                candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )),
    }
}

fn wait_readable(file: &File, timeout: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }

        let timeout_ms = remaining
            .as_nanos()
            .div_ceil(1_000_000)
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
    let device_path = Path::new(device);
    verify_target_hidraw(device_path)?;

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device_path)?;

    // Send only the tested battery-status request.
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

#[cfg(test)]
mod tests {
    use super::{has_battery_report, is_target_hid_id, BATTERY_REPORT_SIGNATURE};

    #[test]
    fn accepts_kernel_zero_padded_usb_hid_id() {
        assert!(is_target_hid_id("0003:00001EA7:00000066"));
    }

    #[test]
    fn accepts_unpadded_hex_fields() {
        assert!(is_target_hid_id("3:1EA7:66"));
    }

    #[test]
    fn rejects_other_devices_and_malformed_ids() {
        assert!(!is_target_hid_id("0003:00001234:00000066"));
        assert!(!is_target_hid_id("0003:00001EA7:00000067"));
        assert!(!is_target_hid_id("0001:00001EA7:00000066"));
        assert!(!is_target_hid_id("0003:00001EA7"));
        assert!(!is_target_hid_id("not-a-hid-id"));
    }

    #[test]
    fn recognizes_the_mouse_battery_report_descriptor() {
        let mut descriptor = vec![0x05, 0x01, 0x09, 0x02];
        descriptor.extend_from_slice(&BATTERY_REPORT_SIGNATURE);
        descriptor.extend_from_slice(&[0x09, 0x02, 0xC0]);

        assert!(has_battery_report(&descriptor));
    }

    #[test]
    fn ignores_descriptors_without_the_vendor_battery_report() {
        let descriptor = [0x05, 0x01, 0x09, 0x02, 0xA1, 0x01, 0xC0];

        assert!(!has_battery_report(&descriptor));
    }
}
