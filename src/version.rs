use chrono::{NaiveDate, Timelike, Utc};

/// Epoch for version calculation: 2020-01-01
const EPOCH: (i32, u32, u32) = (2020, 1, 1);

/// Base36 alphabet (lowercase)
const BASE36_CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// Encode a number to base36 string with specified width (zero-padded)
pub fn encode_base36(mut n: u32, width: usize) -> String {
    if n == 0 {
        return "0".repeat(width);
    }

    let mut result = Vec::new();
    while n > 0 {
        let remainder = (n % 36) as usize;
        result.push(BASE36_CHARS[remainder]);
        n /= 36;
    }

    // Pad with zeros if needed
    while result.len() < width {
        result.push(b'0');
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Decode a base36 string to a number
pub fn decode_base36(s: &str) -> Option<u32> {
    let mut result: u32 = 0;
    for c in s.chars() {
        let digit = match c {
            '0'..='9' => c as u32 - '0' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 10,
            'A'..='Z' => c as u32 - 'A' as u32 + 10,
            _ => return None,
        };
        result = result.checked_mul(36)?.checked_add(digit)?;
    }
    Some(result)
}

/// Generate a version string from the current time
/// Format: DDDMM where DDD = days since epoch, MM = 10-minute slot of day
pub fn generate_version() -> String {
    let now = Utc::now();
    let epoch = NaiveDate::from_ymd_opt(EPOCH.0, EPOCH.1, EPOCH.2).unwrap();
    let today = now.date_naive();

    let days_since_epoch = (today - epoch).num_days() as u32;
    let minutes_since_midnight = now.time().num_seconds_from_midnight() / 60;
    let slot = minutes_since_midnight / 10; // 10-minute slots

    format!(
        "{}{}",
        encode_base36(days_since_epoch, 3),
        encode_base36(slot, 2)
    )
}

/// Parse a version string into (days, slot) components
pub fn parse_version(version: &str) -> Option<(u32, u32)> {
    if version.len() != 5 {
        return None;
    }
    let days = decode_base36(&version[0..3])?;
    let slot = decode_base36(&version[3..5])?;
    Some((days, slot))
}

/// Check if a string is a valid version format
pub fn is_valid_version(s: &str) -> bool {
    s.len() == 5 && s.chars().all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base36() {
        assert_eq!(encode_base36(0, 3), "000");
        assert_eq!(encode_base36(1, 3), "001");
        assert_eq!(encode_base36(35, 3), "00z");
        assert_eq!(encode_base36(36, 3), "010");
        assert_eq!(encode_base36(1000, 3), "0rs");
        assert_eq!(encode_base36(87, 2), "2f");
    }

    #[test]
    fn test_decode_base36() {
        assert_eq!(decode_base36("000"), Some(0));
        assert_eq!(decode_base36("001"), Some(1));
        assert_eq!(decode_base36("00z"), Some(35));
        assert_eq!(decode_base36("010"), Some(36));
        assert_eq!(decode_base36("0rs"), Some(1000));
        assert_eq!(decode_base36("2f"), Some(87));
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        for n in [0, 1, 35, 36, 100, 1000, 10000, 46655] {
            let encoded = encode_base36(n, 3);
            let decoded = decode_base36(&encoded).unwrap();
            assert_eq!(decoded, n, "Failed roundtrip for {}", n);
        }
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0rs2f"), Some((1000, 87)));
        assert_eq!(parse_version("00000"), Some((0, 0)));
        assert_eq!(parse_version("zzz3z"), Some((46655, 143)));
        assert_eq!(parse_version("1234"), None); // Too short
        assert_eq!(parse_version("123456"), None); // Too long
    }

    #[test]
    fn test_is_valid_version() {
        assert!(is_valid_version("1f72f"));
        assert!(is_valid_version("00000"));
        assert!(is_valid_version("zzzzz"));
        assert!(!is_valid_version("1234")); // Too short
        assert!(!is_valid_version("123456")); // Too long
        assert!(!is_valid_version("1f7-f")); // Invalid char
    }

    #[test]
    fn test_generate_version_format() {
        let version = generate_version();
        assert_eq!(version.len(), 5);
        assert!(version.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(is_valid_version(&version));
    }

    #[test]
    fn test_version_ordering() {
        // Versions should be lexicographically sortable
        let v1 = "1f72f";
        let v2 = "1f730";
        let v3 = "1f800";
        assert!(v1 < v2);
        assert!(v2 < v3);
    }
}
