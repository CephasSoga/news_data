use chrono::{Utc, SecondsFormat, Duration as UtcDuration};

/// Returns the time as a string in RFC 3339 format (UTC) with an optional offset stripped.
/// 
/// # Arguments
/// 
/// * `secs` - The number of seconds to subtract from the current UTC time.
/// 
/// # Returns
/// 
/// A `String` representing the UTC time in RFC 3339 format (without the `+00:00` offset).
/// 
/// # Example
/// 
/// ```rust
/// let time = time_rfc3339_opts(3600); // Subtract 1 hour (3600 seconds)
/// println!("{}", time); // Outputs the time 1 hour ago in RFC 3339 format (e.g., "2024-12-16T10:15:30")
/// ```
pub fn time_rfc3339_opts(secs: i64) -> String {
    // Get current UTC time
    let now = Utc::now();
    // Subtract specified seconds from the current time
    let tartget_time = now - UtcDuration::seconds(secs);
    // Format the time in RFC 3339 format with second precision
    let f = tartget_time.to_rfc3339_opts(SecondsFormat::Secs, false);
    // Print the formatted time (for debugging purposes)
    println!("Time f: {}", f);
    // Remove the "+00:00" suffix and return the result
    f.strip_suffix("+00:00").unwrap_or("").to_string()
}

/// Returns the time as a string in the `yyyyMMddTHHmm` format (UTC) with the specified number of seconds subtracted.
/// 
/// # Arguments
/// 
/// * `secs` - The number of seconds to subtract from the current UTC time.
/// 
/// # Returns
/// 
/// A `String` representing the UTC time in the `yyyyMMddTHHmm` format (e.g., "20241216T1030").
/// 
/// # Example
/// 
/// ```rust
/// let time = time_yyyy_mmdd_thhmm(3600); // Subtract 1 hour (3600 seconds)
/// println!("{}", time); // Outputs the time 1 hour ago in yyyyMMddTHHmm format (e.g., "20241216T1030")
/// ```
pub fn time_yyyy_mmdd_thhmm(secs: i64) -> String {
    // Get current UTC time
    let now = Utc::now();
    // Subtract specified seconds from the current time
    let tartget_time = now - UtcDuration::seconds(secs);
    // Format the time in the custom format: yyyyMMddTHHmm
    let f = tartget_time.format("%Y%m%dT%H%M").to_string();
    // Print the formatted time (for debugging purposes)
    println!("Time f: {}", f);
    f
}
