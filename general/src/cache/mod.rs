use std::sync::{Arc, atomic::AtomicUsize};

use moka::sync::{Cache, ConcurrentCacheExt};

use crate::config::YummyConfig;

pub trait YummyCacheResource {
        type K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static;
        type V: Send + Sync + std::clone::Clone + 'static;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }
    fn set(&self, key: Self::K, value: Self::V) -> anyhow::Result<()> { Ok(()) }

}


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

    resource: Box<dyn YummyCacheResource<K = K, V = V>>,

    // Statistical data
    hit: AtomicUsize,
    lose: AtomicUsize,
    miss: AtomicUsize
}

impl<K, V> YummyCache<K, V>
    where 
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static, 
        V: Send + Sync + std::clone::Clone + 'static {

    pub fn new(config: Arc<YummyConfig>, resource: Box<dyn YummyCacheResource::<K = K, V = V>>) -> Self {
        Self {
            cache: Cache::builder().time_to_idle(config.cache_duration.clone()).build(),
            getter: Box::new(default_getter),
            setter: Box::new(default_setter),
            resource,
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
            None => match self.resource.get(key)? {

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
            iter: self.cache.iter()
        } 
    }
}

pub struct YummyCacheIterator<'a, K, V>
    where
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
        V: Send + Sync + std::clone::Clone + 'static {
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


pub trait YummyCacheProperty {
    type K;
    type V;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }
    fn set(&self, key: Self::K, value: Self::V) -> anyhow::Result<()> { Ok(()) }
}

pub struct UserInformationCache {
    cache: YummyCache<String, String>
}

impl YummyCacheProperty for UserInformationCache {
    type K = String;
    type V = String;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>> { Ok(None) }

    fn set(&self, key: Self::K, value: Self::V) -> anyhow::Result<()> { Ok(()) }
}
