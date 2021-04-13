use std::path::PathBuf;
use jm_cache::{CacheFactory, Cache};
use jm_test_utils::sync_helpers::with_tmp_dir;

fn create_cache(dir: PathBuf) -> Cache {
    let cache_factory = CacheFactory::new(dir);
    cache_factory.create_cache("unit_tests").unwrap()
}

#[test]
fn test_cache_miss() {
    with_tmp_dir(|path| {
        let cache = create_cache(path);

        assert_eq!(cache.get("something"), None);
    })
}

#[test]
fn test_cache_hit() {
    with_tmp_dir(|path| {
        let cache = create_cache(path);

        cache.set("package", "something".as_bytes()).unwrap();

        assert_ne!(cache.get("package"), None);
    })
}

#[test]
fn test_cache_key_sanitization() {
    with_tmp_dir(|path| {
        let cache = create_cache(path);


    cache.set("@scope/a", "something".as_bytes()).unwrap();

    let path = cache.get("@scope/a").unwrap();

    assert!(path.to_str().unwrap().contains("@scope_a"));
    })
}
