#![allow(dead_code)]
#![allow(warnings)]
#![allow(unused_variables)]

use std::sync::Arc;
use std::time::Instant;
use std::num::NonZeroUsize;
use std::ops::{Deref, DerefMut};

use lru::LruCache;
use serde_json::Value;
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};


type CacheValue = (Value, Instant);
type LruCacheType = LruCache<String, CacheValue>;


pub enum Lock<'a> {
    Mutex(MutexGuard<'a, LruCache<String, (Value, Instant)>>),
    ReadRwLock(RwLockReadGuard<'a, LruCache<String, (Value, Instant)>>),
    WriteRwLock(RwLockWriteGuard<'a, LruCache<String, (Value, Instant)>>),
}

impl<'a> Lock<'a> {
    pub fn as_mutex(&self) -> Option<&MutexGuard<'a, LruCache<String, (Value, Instant)>>> {
        match self {
            Lock::Mutex(lock) => Some(lock),
            _ => None,
        }
    }
    pub fn as_read_rw_lock(&self) -> Option<&RwLockReadGuard<'a, LruCache<String, (Value, Instant)>>> {
        match self {
            Lock::ReadRwLock(lock) => Some(lock),
            _ => None,
        }
    }
    pub fn as_write_rw_lock(&self) -> Option<&RwLockWriteGuard<'a, LruCache<String, (Value, Instant)>>> {
        match self {
            Lock::WriteRwLock(lock) => Some(lock),
            _ => None,
        }
    }
    
    pub fn lock_get(&mut self, key: &str) -> Option<CacheValue> {
        match self {
            Lock::Mutex(lock) => lock.get(key).cloned(),
            Lock::ReadRwLock(lock) => lock.clone().get(key).cloned(),
            Lock::WriteRwLock(lock) => lock.get(key).cloned(),
        }
    }

    pub fn lock_put(&mut self, key: &str, value: Value) {
        match self {
            Lock::Mutex(lock) => { lock.put(key.to_string(), (value, Instant::now())); },
            Lock::WriteRwLock(lock) => { lock.put(key.to_string(), (value, Instant::now())); },
            Lock::ReadRwLock(_) => {
                panic!("Cannot modify data with a read lock. Acquire a write lock instead.");
            }
        }
    }
}

 
pub trait Cache {
    async fn put(&self, key: String, value: CacheValue);
    async fn get(&self, key: &str) -> Option<CacheValue>;
    async fn pop(&self, key: &str) -> Option<CacheValue>;
    async fn lock_read(&mut self) -> Lock;
    async fn lock_write(&mut self) -> Lock<'_>;  
}

/// SharedCache with Mutex for async single-threaded scenarios.
pub struct SharedCache {
    inner: Arc<Mutex<LruCacheType>>,
}

impl SharedCache {
    pub fn new(capacity: usize) -> Self {
        SharedCache {
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
        }
    }

    async fn lock(&mut self) -> Lock<'_> {
        Lock::Mutex(self.inner.lock().await)
    } 
}

impl  Cache for SharedCache{
    async fn put(&self, key: String, value: CacheValue) {
        let mut cache = self.inner.lock().await; // Async lock
        cache.put(key, value);
    }

    async fn get(&self, key: &str) -> Option<CacheValue> {
        let mut cache = self.inner.lock().await; // Async lock
        cache.get(key).cloned()
    }

    async fn pop(&self, key: &str) -> Option<CacheValue> {
        let mut cache = self.inner.lock().await; // Async lock for write access
        cache.pop(key)
    }

    async fn lock_read(&mut self) -> Lock {
        self.lock().await
    }

    async fn lock_write(&mut self) -> Lock<'_> {
        self.lock().await
    }

}

/// SharedLockedCache with Read for read-heavy async scenarios.
pub struct SharedLockedCache {
    inner: Arc<RwLock<LruCacheType>>,
}

impl SharedLockedCache {
    pub fn new(capacity: usize) -> Self {
        SharedLockedCache {
            inner: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
        }
    }
}

impl Cache for SharedLockedCache {
    async fn put(&self, key: String, value: CacheValue) {
        let mut cache = self.inner.write().await; // Async write lock
        cache.put(key, value);
    }

    async fn get(&self, key: &str) -> Option<CacheValue> {
        let cache = self.inner.read().await; // Async read lock
        cache.clone().get(key).cloned()
    }

    async fn pop(&self, key: &str) -> Option<CacheValue> {
        let mut cache = self.inner.write().await; // Async write lock for removal
        cache.pop(key)
    }

    async fn lock_read(&mut self) -> Lock {
        Lock::ReadRwLock(self.inner.read().await)
    }

    async fn lock_write(&mut self) -> Lock<'_> {
        Lock::WriteRwLock(self.inner.write().await)
    }

}

impl Deref for SharedLockedCache {
    type Target = Arc<RwLock<LruCacheType>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SharedLockedCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}