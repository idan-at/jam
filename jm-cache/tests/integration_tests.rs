use jm_cache::Cache;

fn create_cache() -> Cache {
  Cache::new("testing").unwrap()
}

#[test]
fn test_cache_miss() {
  let cache = create_cache();

  assert_eq!(cache.get("something"), None);
}

#[test]
fn test_cache_hit() {
  let cache = create_cache();

  cache.set("package", "something".to_string()).unwrap();

  assert_ne!(cache.get("package"), None);
}

#[test]
fn test_cache_key_sanitization() {
  let cache = create_cache();

  cache.set("@scope/a", "something".to_string()).unwrap();

  let path = cache.get("@scope/a").unwrap();

  assert!(path.to_str().unwrap().contains("@scope_a"));
}
