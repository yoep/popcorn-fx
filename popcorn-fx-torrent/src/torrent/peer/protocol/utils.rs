use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current system time as UNIX timestamp in micro seconds.
pub fn now_as_micros() -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));
    (now.as_micros() & 0xffff_ffff) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_as_micros() {
        let result = now_as_micros();
        assert_ne!(0, result, "expected the current timestamp in micros");
    }
}
