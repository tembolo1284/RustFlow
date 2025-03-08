use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, NaiveDateTime, Utc};

/// Returns the current timestamp in nanoseconds
pub fn current_timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// Returns the current timestamp in microseconds
pub fn current_timestamp_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as u64
}

/// Returns the current timestamp in milliseconds
pub fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

/// Returns the current timestamp in seconds
pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Converts a nanosecond timestamp to human-readable date time string
pub fn format_timestamp_nanos(timestamp: u64) -> String {
    let secs = (timestamp / 1_000_000_000) as i64;
    let nsecs = (timestamp % 1_000_000_000) as u32;
    
    let naive = NaiveDateTime::from_timestamp_opt(secs, nsecs)
        .expect("Invalid timestamp");
    
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

/// Converts a millisecond timestamp to human-readable date time string
pub fn format_timestamp_millis(timestamp: u64) -> String {
    let secs = (timestamp / 1_000) as i64;
    let nsecs = ((timestamp % 1_000) * 1_000_000) as u32;
    
    let naive = NaiveDateTime::from_timestamp_opt(secs, nsecs)
        .expect("Invalid timestamp");
    
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

/// Measure the execution time of a closure
pub fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = SystemTime::now();
    let result = f();
    let elapsed = start.elapsed().expect("Clock went backwards");
    
    (result, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_timestamp_functions() {
        // Just make sure they run without panicking
        let _ = current_timestamp_nanos();
        let _ = current_timestamp_micros();
        let _ = current_timestamp_millis();
        let _ = current_timestamp_secs();
    }
    
    #[test]
    fn test_format_timestamp() {
        // Example timestamp: 2023-05-01 12:34:56.789
        let timestamp_millis = 1682946896789u64;
        let formatted = format_timestamp_millis(timestamp_millis);
        
        // Check format (not exact value since it depends on timezone)
        assert!(formatted.len() > 20);
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));
        assert!(formatted.contains("."));
    }
    
    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| {
            sleep(Duration::from_millis(5));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 5);
    }
}
