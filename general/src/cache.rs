use std::{sync::{Arc, atomic::AtomicUsize}, marker::PhantomData};

use moka::sync::{Cache, ConcurrentCacheExt};

use crate::config::YummyConfig;

pub type YummyCacheGetter<K, V> = fn(&K) -> anyhow::Result<Option<V>>;

pub struct YummyCache<K, V>
    where
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
        V: Send + Sync + std::clone::Clone + 'static {

    // Cache holder
    cache: Cache<K, V>,

    // Fetch information from resource callback
    getter: YummyCacheGetter<K, V>,

    // Statistical data
    hit: AtomicUsize,
    lose: AtomicUsize,
    miss: AtomicUsize
}

impl<K, V> YummyCache<K, V>
    where 
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static, 
        V: Send + Sync + std::clone::Clone + 'static {

    pub fn new(config: Arc<YummyConfig>, getter: YummyCacheGetter<K, V>) -> Self {
        Self {
            cache: Cache::builder().time_to_idle(config.cache_duration.clone()).build(),
            getter,
            hit: AtomicUsize::new(0),
            lose: AtomicUsize::new(0),
            miss: AtomicUsize::new(0)
        }
    }

    pub fn get(&self, key: &K) -> anyhow::Result<Option<V>> {

        // Try to get information from cache
        match self.cache.get(key) {

            // We found the cache
            Some(result) => {
                // Increase hit information
                self.hit.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Ok(Some(result))
            },

            // We dont have information in the cache. lets fetch it from resource
            None => match (self.getter)(key)? {

                // Information is in the resource and lets save it into the cache
                Some(value) => {

                    // We dont have cache, so, lets increase lose information
                    self.lose.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Save information
                    self.set(key.clone(), value.clone());

                    // Return information
                    Ok(Some(value))
                },

                // The information is not the resource
                None => {
                    
                    // We try to access missing information, increase the miss counter
                    self.miss.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    Ok(None)
                }
            }
        }
    }

    pub fn set(&self, key: K, value: V) {
        self.cache.insert(key, value)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    pub fn remove(&self, key: &K) {
        self.cache.invalidate(key)
    }

    pub fn sync(&self) {
        self.cache.sync()
    }

    pub fn get_hit(&self) -> usize {
        self.hit.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_lose(&self) -> usize {
        self.lose.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_miss(&self) -> usize {
        self.miss.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn iter(&self) -> YummyCacheIterator<K, V> {
        YummyCacheIterator {
            _marker1: PhantomData,
            _marker2: PhantomData,
            iter: self.cache.iter()
        } 
    }
}

pub struct YummyCacheIterator<'a, K, V>
    where
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
        V: Send + Sync + std::clone::Clone + 'static {
    _marker1: PhantomData<K>,
    _marker2: PhantomData<V>,
    iter: moka::sync::Iter<'a, K, V>
}

impl<'a, K, V> Iterator for YummyCacheIterator<'a, K, V> where
    K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
    V: Send + Sync + std::clone::Clone + 'static {
    type Item = (Arc<K>, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
