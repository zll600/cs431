//! Thread-safe key/value cache.

use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};

/// Cache that remembers the result for each key.
#[derive(Debug, Default)]
pub struct Cache<K, V> {
    // todo! This is an example cache type. Build your own cache type that satisfies the
    // specification for `get_or_insert_with`.
    inner: Arc<RwLock<HashMap<K, Option<V>>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    /// Retrieve the value or insert a new one created by `f`.
    ///
    /// An invocation to this function should not block another invocation with a different key.
    /// For example, if a thread calls `get_or_insert_with(key1, f1)` and another thread calls
    /// `get_or_insert_with(key2, f2)` (`key1≠key2`, `key1,key2∉cache`) concurrently, `f1` and `f2`
    /// should run concurrently.
    ///
    /// On the other hand, since `f` may consume a lot of resource (= money), it's desirable not to
    /// duplicate the work. That is, `f` should be run only once for each key. Specifically, even
    /// for the concurrent invocations of `get_or_insert_with(key, f)`, `f` is called only once.
    pub fn get_or_insert_with<F: FnOnce(K) -> V>(&self, key: K, f: F) -> V {
        let table = (*self.inner).read().unwrap();
        let is_exist: bool = table.contains_key(&key);
        drop(table);

        // value for given key is stored or being calculated by other thread
        if is_exist {
            loop {
                let table = (*self.inner).read().unwrap();
                let value_status = table.get(&key).unwrap();
                if let Some(v) = value_status {
                    break;
                }
            }
            let table = (*self.inner).read().unwrap();
            let result = table.get(&key).unwrap().as_ref().unwrap().clone();
            result
        }
        // value for given key is neither being calculated nor stored in map
        else {
            let mut table_write = (*self.inner).write().unwrap();

            // prevent two threads writing k-v at the same time
            if table_write.contains_key(&key) {
                drop(table_write);
                loop {
                    let table = (*self.inner).read().unwrap();
                    let value_status = table.get(&key).unwrap();
                    if let Some(v) = value_status {
                        break;
                    }
                }
                let table = (*self.inner).read().unwrap();
                let result = table.get(&key).unwrap().as_ref().unwrap().clone();
                result
            } else {
                table_write.insert(key.clone(), None);
                drop(table_write);

                let key_copy = key.clone();
                let result = f(key);

                let mut table_write = (*self.inner).write().unwrap();
                table_write.insert(key_copy, Some(result.clone()));

                result
            }
        }
    }
}
