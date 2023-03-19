/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::sync::{Arc, atomic::AtomicUsize};

use yummy_model::config::YummyConfig;
use moka::sync::{Cache, ConcurrentCacheExt};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
pub struct YummyCache<K, V>
    where
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
        V: Send + Sync + std::clone::Clone + 'static {

    // Cache holder
    cache: Cache<K, V>,

    // Fetch information from resource callback
    resource: Box<dyn YummyCacheResource<K = K, V = V>>,

    // Statistical data
    hit: AtomicUsize,
    lose: AtomicUsize,
    miss: AtomicUsize,
    insert: AtomicUsize,
    delete: AtomicUsize
}

pub struct YummyCacheIterator<'a, K, V>
    where
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
        V: Send + Sync + std::clone::Clone + 'static {
    iter: moka::sync::Iter<'a, K, V>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */
pub trait YummyCacheResource {
    type K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static;
    type V: Send + Sync + std::clone::Clone + 'static;

    fn get(&self, key: &Self::K) -> anyhow::Result<Option<Self::V>>;
    fn set(&self, _: &Self::K, _: &Self::V) -> anyhow::Result<()> { Ok(()) }
}

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<K, V> YummyCache<K, V>
    where 
        K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static, 
        V: Send + Sync + std::clone::Clone + 'static {

    pub fn new(config: Arc<YummyConfig>, resource: Box<dyn YummyCacheResource::<K = K, V = V>>) -> Self {
        Self {
            cache: Cache::builder().time_to_idle(config.cache_duration).build(),
            resource,
            hit: AtomicUsize::new(0),
            lose: AtomicUsize::new(0),
            miss: AtomicUsize::new(0),
            insert: AtomicUsize::new(0),
            delete: AtomicUsize::new(0)
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
                    self.set(key, value.clone())?;

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

    pub fn set(&self, key: &K, value: V) -> anyhow::Result<()> {
        // Increase insert counter
        self.insert.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // First change resource
        self.resource.set(key, &value)?;

        // If the resource update success, update on the cache
        self.cache.insert(key.clone(), value);
        Ok(())
    }

    pub fn contains(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    pub fn remove(&self, key: &K) {
        // Increase delete counter
        self.delete.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

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
            iter: self.cache.iter()
        } 
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl<'a, K, V> Iterator for YummyCacheIterator<'a, K, V> where
    K: Send + Sync + std::clone::Clone + std::hash::Hash + std::cmp::Eq + 'static,
    V: Send + Sync + std::clone::Clone + 'static {
    type Item = (Arc<K>, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
