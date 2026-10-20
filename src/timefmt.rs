//! Timestamp formatting in a configured IANA timezone (default: Mexico City).

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;

use crate::error::{Error, Result};

pub fn parse_timezone(name: &str) -> Result<Tz> {
    name.parse::<Tz>()
        .map_err(|e| Error::Config(format!("timezone {name:?}: {e}")))
}

/// Format unix epoch millis as `HH:MM:SS` in `tz_name` (IANA).
/// Falls back to America/Mexico_City, then UTC, if the name is invalid.
pub fn format_hhmmss(ts_ms: u64, tz_name: &str) -> String {
    let tz = parse_timezone(tz_name)
        .ok()
        .or_else(|| "America/Mexico_City".parse::<Tz>().ok())
        .unwrap_or(chrono_tz::UTC);

    let Some(utc) = Utc.timestamp_millis_opt(ts_ms as i64).single() else {
        return "--:--:--".into();
    };
    let local = utc.with_timezone(&tz);
    local.format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mexico_city_is_utc_minus_6() {
        // 2024-01-15 18:00:00 UTC → 12:00:00 in Mexico City (no DST since 2022)
        let ts_ms = 1_705_341_600_000u64;
        let s = format_hhmmss(ts_ms, "America/Mexico_City");
        assert_eq!(s, "12:00:00", "got {s}");
    }

    #[test]
    fn parses_mexico_city() {
        assert!(parse_timezone("America/Mexico_City").is_ok());
    }
}
