#![allow(dead_code)]

use std::sync::Arc;
use std::time::{Instant, Duration, SystemTime};

use rand::{thread_rng, Rng};
use chrono::{Utc, SecondsFormat, DateTime, Duration as UtcDuration};
use futures_util::Future;
use tokio::time::sleep;
use serde_json::Value;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

use crate::cache::{Cache, SharedLockedCache};
use crate::config::ValueConfig;


pub fn time_rfc3339_opts(secs: i64) -> String {
    // Get current UTC time
    let now = Utc::now();
    // Subtract specified seconds from the current time
    let tartget_time = now - UtcDuration::seconds(secs);
    // Format the time in RFC 3339 format with second precision
    let f = tartget_time.to_rfc3339_opts(SecondsFormat::Secs, false);
    // Print the formatted time (for debugging purposes)
    info!("Action at Time f: {}", f);
    // Remove the "+00:00" suffix and return the result
    f.strip_suffix("+00:00").unwrap_or("").to_string()
}

pub fn time_yyyy_mmdd_thhmm(secs: i64) -> String {
    // Get current UTC time
    let now = Utc::now();
    // Subtract specified seconds from the current time
    let tartget_time = now - UtcDuration::seconds(secs);
    // Format the time in the custom format: yyyyMMddTHHmm
    let f = tartget_time.format("%Y%m%dT%H%M").to_string();
    // Print the formatted time (for debugging purposes)
    info!("Action at Time f: {}", f);
    f
}

pub  fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}


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

pub async fn retry<F, Fut, T, E>(
    config: &Arc<ValueConfig>,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;

    loop {
        attempts += 1;
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) if attempts < config.task.max_retries => {
                warn!("Attempt {}/{} failed with error: {:?}.", &attempts, &config.task.max_retries, err);
                debug!("Attempting again...");
                let delay = std::cmp::min(
                    config.task.base_delay_ms * (2u32.pow(attempts - 1)),
                    config.task.max_delay_ms,
                );
                sleep(Duration::from_millis(delay as u64)).await;
            }
            Err(err) => {
                error!("All {} attempts have been unsuccessful. | Returning final error. | Error: {:?}", &config.task.max_retries, err);
                return Err(err)
            },
        }
    }
}


pub async fn get_from_cache_or_fetch<F, Fut>(
    cache: &Arc<Mutex<SharedLockedCache>>,
    key: &str,
    fetch_fn: F,
    ttl: u32,
) -> Result<Value, reqwest::Error>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Value, reqwest::Error>>,
{
    info!("Looking in cache for {}...", &key);
    let cache = cache.lock().await;
    if let Some((value, instant)) = cache.get(key).await {
        info!("Found in cache");
        if instant.elapsed() < Duration::from_secs(ttl as u64) {
            info!("Target data found in cache.");
            return Ok(value.clone());
        } else {
            warn!("Expired key: {}. Removing...", &key);
            cache.pop(key).await; // Expired
        }
    }
    info!("Target not found in cache. | HTTP GET requested the data...");
    // Fetch and cache the value
    let result = fetch_fn().await;
    match result {
        Ok(value) => {
            info!("Got value: {:?}", !value.is_null());
            cache.put(key.to_string(), (value.clone(), Instant::now())).await;
            Ok(value)
        }
        Err(e) => {
            error!("Error for GET request: {}", e);
            Err(e)
        },
    }
}