use std::{sync::{Arc, atomic::AtomicUsize}, marker::PhantomData};

use moka::sync::{Cache, ConcurrentCacheExt};

use crate::config::YummyConfig;

pub type YummyCacheGetter<K, V> = dyn Fn(&K) -> anyhow::Result<Option<V>>;
pub type YummyCacheSetter<K, V> = dyn Fn(&K, &V) -> anyhow::Result<()>;

fn default_getter<K, V>(_: &K) -> anyhow::Result<Option<V>>
where 
    K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static, 
    V: Send + Sync + std::clone::Clone + 'static { Ok(None) }

fn default_setter<K, V>(_: &K, _: &V) -> anyhow::Result<()>
where 
    K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static, 
    V: Send + Sync + std::clone::Clone + 'static { Ok(()) }

pub struct YummyCache<K, V>
    where
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
        V: Send + Sync + std::clone::Clone + 'static {

    // Cache holder
    cache: Cache<K, V>,

    // Fetch information from resource callback
    getter: Box<YummyCacheGetter<K, V>>,
    setter: Box<YummyCacheSetter<K, V>>,

    // Statistical data
    hit: AtomicUsize,
    lose: AtomicUsize,
    miss: AtomicUsize
}

impl<K, V> YummyCache<K, V>
    where 
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static, 
        V: Send + Sync + std::clone::Clone + 'static {

    pub fn new(config: Arc<YummyConfig>) -> Self {
        Self {
            cache: Cache::builder().time_to_idle(config.cache_duration.clone()).build(),
            getter: Box::new(default_getter),
            setter: Box::new(default_setter),
            hit: AtomicUsize::new(0),
            lose: AtomicUsize::new(0),
            miss: AtomicUsize::new(0)
        }
    }

    pub fn execute_get(&self, key: &K, resource_getter: &YummyCacheGetter<K, V>) -> anyhow::Result<Option<V>> {

        // Try to get information from cache
        match self.cache.get(key) {

            // We found the cache
            Some(result) => {
                // Increase hit information
                self.hit.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Ok(Some(result))
            },

            // We dont have information in the cache. lets fetch it from resource
            None => match (resource_getter)(key)? {

                // Information is in the resource and lets save it into the cache
                Some(value) => {

                    // We dont have cache, so, lets increase lose information
                    self.lose.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Save information
                    self.set(key.clone(), value.clone())?;

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

    pub fn get(&self, key: &K) -> anyhow::Result<Option<V>> {
        self.execute_get(key, &self.getter)
    }

    pub fn execute_set(&self, key: K, value: V, resource_setter: &YummyCacheSetter<K, V>) -> anyhow::Result<()> {
        // First change resource
        (resource_setter)(&key, &value)?;

        // If the resource update success, update on the cache
        self.cache.insert(key, value);
        Ok(())
    }

    pub fn set(&self, key: K, value: V) -> anyhow::Result<()> {
        self.execute_set(key, value, &self.setter)
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

    pub fn set_getter(&mut self, getter: &'static YummyCacheGetter<K, V>) {
        self.getter = Box::new(getter);
    }

    pub fn set_setter(&mut self, setter: &'static YummyCacheSetter<K, V>) {
        self.setter = Box::new(setter);
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
