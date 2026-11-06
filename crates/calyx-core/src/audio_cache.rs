use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct AudioCache {
    pub dir: PathBuf,
    pub max_bytes: u64,
}

impl AudioCache {
    pub fn default_dir() -> PathBuf {
        crate::paths::audio_cache_dir()
    }

    pub fn new(max_bytes: u64) -> Self {
        Self {
            dir: AudioCache::default_dir(),
            max_bytes,
        }
    }

    pub fn with_dir(dir: PathBuf, max_bytes: u64) -> Self {
        Self { dir, max_bytes }
    }

    fn safe_id(id: &str) -> String {
        id.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect()
    }

    pub fn get(&self, track_id: &str) -> Option<Vec<u8>> {
        let file = self.dir.join(format!("{}.bin", AudioCache::safe_id(track_id)));
        if !file.exists() {
            return None;
        }
        let _ = fs::OpenOptions::new().write(true).open(&file).and_then(|f| f.set_modified(SystemTime::now()));
        fs::read(&file).ok()
    }

    pub fn put(&self, track_id: &str, data: &[u8]) -> std::io::Result<()> {
        let safe_id = AudioCache::safe_id(track_id);
        let tmp_dir = self.dir.join("tmp");
        fs::create_dir_all(&tmp_dir)?;

        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let part_path = tmp_dir.join(format!("{}.part_{}", safe_id, ts));
        fs::write(&part_path, data)?;
        fs::rename(part_path, self.dir.join(format!("{}.bin", safe_id)))?;
        Ok(())
    }

    pub fn contains(&self, track_id: &str) -> bool {
        self.dir.join(format!("{}.bin", AudioCache::safe_id(track_id))).exists()
    }

    pub fn current_size(&self) -> u64 {
        let mut size = 0;
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("bin")
                    && let Ok(meta) = entry.metadata() {
                        size += meta.len();
                    }
            }
        }
        size
    }

pub fn evict_if_needed(&self) -> usize {
    let Ok(entries) = fs::read_dir(&self.dir) else { return 0; };
    let mut files: Vec<(SystemTime, u64, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("bin") {
                return None;
            }
            let meta = e.metadata().ok()?;
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            Some((mtime, meta.len(), path))
        })
        .collect();

    files.sort_by_key(|&(m, _, _)| m);
    let mut total = self.current_size();
    let mut evicted = 0;
    let target = self.max_bytes.saturating_mul(75) / 100;

    for (_, len, path) in &files {
        if total <= target { break; }
        if fs::remove_file(path).is_ok() {
            evicted += 1;
            total = total.saturating_sub(*len);
        }
    }

    let tmp_dir = self.dir.join("tmp");
    if let Ok(entries) = fs::read_dir(&tmp_dir) {
        let now = SystemTime::now();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = entry.metadata()
                && let Ok(mtime) = meta.modified()
                    && now.duration_since(mtime).map(|d| d.as_secs() > 3600).unwrap_or(false) {
                        fs::remove_file(path).ok();
                    }
        }
    }

    evicted
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_put_and_get() {
        let dir = std::env::temp_dir().join(format!("calyx_test_cache_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let cache = AudioCache::with_dir(dir.clone(), 1024);

        assert!(cache.put("test_track_1", b"hello audio data").is_ok());
        assert_eq!(cache.get("test_track_1"), Some(b"hello audio data".to_vec()));
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
        std::fs::OpenOptions::new().write(true).open(&file).unwrap().set_modified(old).unwrap();

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
}