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

pub fn parse_response(data: &[u8]) -> ParseResult {
    if data.first() != Some(&REPORT_ID) || data.get(1) != Some(&BATTERY_COMMAND) {
        return ParseResult::Unrelated;
    }

    if data.len() < REPORT_LEN {
        return ParseResult::Invalid("battery response is shorter than 8 bytes");
    }

    let percent = data[3];
    if percent > 100 {
        return ParseResult::Invalid("battery percentage is outside 0..=100");
    }

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

    #[test]
    fn parses_observed_battery_response() {
        let response = [0xB5, 0x01, 0x01, 0x38, 0x2E, 0xFF, 0x09, 0x70];

        assert_eq!(
            parse_response(&response),
            ParseResult::Battery(BatteryResponse {
                status: 0x01,
                percent: 56,
                raw: response,
            })
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
    fn rejects_out_of_range_battery_values() {
        let response = [0xB5, 0x01, 0x01, 101, 0, 0, 0, 0];

        assert_eq!(
            parse_response(&response),
            ParseResult::Invalid("battery percentage is outside 0..=100")
        );
    }

    #[test]
    fn maps_battery_percentage_to_waybar_states() {
        assert_eq!(battery_class(100), "normal");
        assert_eq!(battery_class(31), "normal");
        assert_eq!(battery_class(30), "warning");
        assert_eq!(battery_class(16), "warning");
        assert_eq!(battery_class(15), "critical");
        assert_eq!(battery_class(0), "critical");
    }
}
