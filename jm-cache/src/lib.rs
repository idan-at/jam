pub mod errors;

use dashmap::DashMap;
use errors::JmCacheError;

pub struct Cache<T> {
    cache: DashMap<String, T>,
}

// impl Cache<T> {
//     pub fn new(): Result<Cache<T>, JmCacheError> {}
// }
