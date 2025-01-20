use rand::{thread_rng, Rng};
use chrono::{Utc, SecondsFormat, Duration as UtcDuration};

/// Returns the time as a string in RFC 3339 format (UTC) with an optional offset stripped.
/// 
/// ## Arguments
/// 
/// * `secs` - The number of seconds to subtract from the current UTC time.
/// 
/// ## Returns
/// 
/// A `String` representing the UTC time in RFC 3339 format (without the `+00:00` offset).
/// 
/// ## Example
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
    println!("Action at Time f: {}", f);
    // Remove the "+00:00" suffix and return the result
    f.strip_suffix("+00:00").unwrap_or("").to_string()
}

/// Returns the time as a string in the `yyyyMMddTHHmm` format (UTC) with the specified number of seconds subtracted.
/// 
/// ## Arguments
/// 
/// * `secs` - The number of seconds to subtract from the current UTC time.
/// 
/// ## Returns
/// 
/// A `String` representing the UTC time in the `yyyyMMddTHHmm` format (e.g., "20241216T1030").
/// 
/// ## Example
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
    println!("Action at Time f: {}", f);
    f
}

/// Returns the current time in RFC 3339 format (to seconds precision).
///
/// The time is provided in UTC without the timezone offset (`+00:00`).
///
/// # Example
/// ```
/// let current_time = now();
/// println!("Current time: {}", current_time);
/// ```
///
/// Output example: `2024-12-16T10:15:30Z`
///
/// # Returns
/// A `String` containing the current UTC time formatted according to RFC 3339.
pub  fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

/// Generates a random alphanumeric string of the specified length.
///
/// The generated string includes uppercase letters, lowercase letters, and digits.
///
/// # Arguments
/// * `length` - The length of the random key to generate.
///
/// # Example
/// ```
/// let random_key = generate_random_key(8);
/// println!("Random key: {}", random_key);
/// ```
///
/// Output example: `aB3dE7Fg`
///
/// # Returns
/// A `String` containing a random alphanumeric key.
///
/// # Panics
/// This function does not panic.
pub fn generate_random_key(length: usize) -> String {
    let mut rng = thread_rng();
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"; // Alphanumeric charset

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0, charset.len());
            char::from_u32(charset[idx] as u32).unwrap_or('0')
        })
        .collect()
}