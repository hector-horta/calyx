use super::*;

#[test]
fn test_atomic_put_and_get() {
    let dir = std::env::temp_dir().join(format!("calyx_test_cache_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let cache = AudioCache::with_dir(dir.clone(), 1024);

    assert!(cache.put("test_track_1", b"hello audio data").is_ok());
    assert_eq!(
        cache.get("test_track_1"),
        Some(b"hello audio data".to_vec())
    );
    assert!(cache.contains("test_track_1"));
    assert!(!cache.contains("non_existent"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn get_refreshes_mtime_for_lru() {
    let dir = std::env::temp_dir().join(format!("calyx_test_touch_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let cache = AudioCache::with_dir(dir.clone(), 1024);
    cache.put("t1", b"abc").unwrap();

    let file = dir.join("t1.bin");

    // Age the file so a refreshed mtime is detectable.
    let old = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(old)
        .unwrap();

    assert!(cache.get("t1").is_some());

    let after = std::fs::metadata(&file).unwrap().modified().unwrap();
    assert!(after > old, "get() must refresh mtime so eviction is LRU");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_eviction_when_exceeding_quota() {
    let dir = std::env::temp_dir().join(format!("calyx_test_evict_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let cache = AudioCache::with_dir(dir.clone(), 100);

    let data = vec![0u8; 60];
    assert!(cache.put("track_a", &data).is_ok());
    std::thread::sleep(std::time::Duration::from_millis(15));
    assert!(cache.put("track_b", &data).is_ok());

    assert!(cache.current_size() >= 120);
    let evicted = cache.evict_if_needed();
    assert!(evicted >= 1);
    assert!(cache.current_size() <= 75);

    let _ = fs::remove_dir_all(&dir);
}
