pub const REPORT_ID: u8 = 0xB5;
pub const BATTERY_COMMAND: u8 = 0x01;
pub const REPORT_LEN: usize = 8;

pub const BATTERY_REQUEST: [u8; REPORT_LEN] = [REPORT_ID, BATTERY_COMMAND, 0, 0, 0, 0, 0, 0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryResponse {
    pub status: u8,
    pub percent: u8,
    pub raw: [u8; REPORT_LEN],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResult {
    Unrelated,
    Invalid(&'static str),
    Battery(BatteryResponse),
}

/// Decode the packed-BCD battery field observed in byte 7 of the tested
/// ONE MAX OM-W45B status response.
///
/// Observed hardware samples:
/// - 0x00 with the battery removed
/// - 0x30 with a weak/old battery
/// - 0x70 with a nearly new battery
///
/// This is an evidence-based interpretation for the tested receiver, not a
/// documented protocol guarantee. A single packed-BCD byte represents 00..99.
pub fn decode_bcd_percent(value: u8) -> Option<u8> {
    let tens = value >> 4;
    let ones = value & 0x0F;

    if tens > 9 || ones > 9 {
        return None;
    }

    Some(tens * 10 + ones)
}

pub fn parse_response(data: &[u8]) -> ParseResult {
    if data.first() != Some(&REPORT_ID) || data.get(1) != Some(&BATTERY_COMMAND) {
        return ParseResult::Unrelated;
    }

    if data.len() < REPORT_LEN {
        return ParseResult::Invalid("battery response is shorter than 8 bytes");
    }

    // Byte 3 was previously misidentified as the percentage: it remained
    // 0x38 in all user tests regardless of battery state. Byte 7 tracks the
    // observed battery states and is decoded as packed BCD instead.
    let Some(percent) = decode_bcd_percent(data[7]) else {
        return ParseResult::Invalid("battery field is not valid packed BCD (00..99)");
    };

    let mut raw = [0_u8; REPORT_LEN];
    raw.copy_from_slice(&data[..REPORT_LEN]);

    ParseResult::Battery(BatteryResponse {
        status: data[2],
        percent,
        raw,
    })
}

pub fn battery_class(percent: u8) -> &'static str {
    match percent {
        0..=15 => "critical",
        16..=30 => "warning",
        _ => "normal",
    }
}

pub fn format_hex(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_response(battery_field: u8) -> [u8; REPORT_LEN] {
        [0xB5, 0x01, 0x01, 0x38, 0x2E, 0xFF, 0x09, battery_field]
    }

    #[test]
    fn decodes_observed_bcd_battery_states() {
        for (raw_bcd, expected_percent) in [(0x00, 0), (0x30, 30), (0x70, 70)] {
            assert_eq!(decode_bcd_percent(raw_bcd), Some(expected_percent));
        }
    }

    #[test]
    fn parses_battery_percentage_from_byte_seven_not_byte_three() {
        let response = sample_response(0x70);

        assert_eq!(
            parse_response(&response),
            ParseResult::Battery(BatteryResponse {
                status: 0x01,
                percent: 70,
                raw: response,
            })
        );
    }

    #[test]
    fn parses_zero_and_weak_battery_samples() {
        let no_battery = sample_response(0x00);
        let weak_battery = sample_response(0x30);

        assert!(matches!(
            parse_response(&no_battery),
            ParseResult::Battery(BatteryResponse { percent: 0, .. })
        ));
        assert!(matches!(
            parse_response(&weak_battery),
            ParseResult::Battery(BatteryResponse { percent: 30, .. })
        ));
    }

    #[test]
    fn rejects_invalid_bcd_nibbles() {
        assert_eq!(decode_bcd_percent(0xA0), None);
        assert_eq!(decode_bcd_percent(0x7A), None);
        assert_eq!(
            parse_response(&sample_response(0xFA)),
            ParseResult::Invalid("battery field is not valid packed BCD (00..99)")
        );
    }

    #[test]
    fn ignores_regular_mouse_reports() {
        let response = [0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];

        assert_eq!(parse_response(&response), ParseResult::Unrelated);
    }

    #[test]
    fn rejects_short_battery_reports() {
        let response = [0xB5, 0x01, 0x01, 0x38];

        assert_eq!(
            parse_response(&response),
            ParseResult::Invalid("battery response is shorter than 8 bytes")
        );
    }

    #[test]
    fn maps_battery_percentage_to_waybar_states() {
        assert_eq!(battery_class(99), "normal");
        assert_eq!(battery_class(31), "normal");
        assert_eq!(battery_class(30), "warning");
        assert_eq!(battery_class(16), "warning");
        assert_eq!(battery_class(15), "critical");
        assert_eq!(battery_class(0), "critical");
    }
}
