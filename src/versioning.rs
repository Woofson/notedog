use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version_id: String,
    pub timestamp_sec: u64,
    pub timestamp_nanos: u32,
    pub formatted_time: String,
    pub note_path: PathBuf,
    pub version_path: PathBuf,
    pub size_bytes: usize,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct VersionManager {
    pub root_versions_dir: PathBuf,
}

impl VersionManager {
    pub fn new(note_folder: &Path) -> Self {
        let root_versions_dir = note_folder.join(".notedog_versions");
        if !root_versions_dir.exists() {
            let _ = fs::create_dir_all(&root_versions_dir);
        }
        Self { root_versions_dir }
    }

    pub fn get_version_dir_for_note(&self, note_path: &Path) -> PathBuf {
        let relative = note_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let parent = note_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string());

        let grand = note_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string());

        self.root_versions_dir.join(grand).join(parent).join(relative)
    }

    pub fn create_snapshot(&self, note_path: &Path, content: &[u8]) -> io::Result<PathBuf> {
        let version_dir = self.get_version_dir_for_note(note_path);
        fs::create_dir_all(&version_dir)?;

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();

        // Check if identical to last version to prevent duplicate zero-change snapshots
        let existing = self.list_versions(note_path)?;
        if let Some(latest) = existing.first() {
            if let Ok(latest_bytes) = fs::read(&latest.version_path) {
                if latest_bytes == content {
                    return Ok(latest.version_path.clone());
                }
            }
        }

        let formatted = format_timestamp(secs);
        let filename = format!("{}_{}_{}.rev", secs, nanos, formatted.replace(':', "-").replace(' ', "_"));
        let target_path = version_dir.join(filename);

        fs::write(&target_path, content)?;
        Ok(target_path)
    }

    pub fn list_versions(&self, note_path: &Path) -> io::Result<Vec<VersionInfo>> {
        let version_dir = self.get_version_dir_for_note(note_path);
        let mut result = Vec::new();

        if !version_dir.exists() {
            return Ok(result);
        }

        if let Ok(entries) = fs::read_dir(&version_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "rev") {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    let parts: Vec<&str> = filename.split('_').collect();
                    let ts_sec: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                    let ts_nanos: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let formatted_time = format_timestamp(ts_sec);
                    let size_bytes = fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0);
                    let is_encrypted = note_path.extension().map_or(false, |ext| ext == "enc");

                    result.push(VersionInfo {
                        version_id: filename.to_string(),
                        timestamp_sec: ts_sec,
                        timestamp_nanos: ts_nanos,
                        formatted_time,
                        note_path: note_path.to_path_buf(),
                        version_path: path,
                        size_bytes,
                        is_encrypted,
                    });
                }
            }
        }

        result.sort_by(|a, b| {
            b.timestamp_sec
                .cmp(&a.timestamp_sec)
                .then_with(|| b.timestamp_nanos.cmp(&a.timestamp_nanos))
        });
        Ok(result)
    }

    pub fn restore_version(&self, version: &VersionInfo) -> io::Result<()> {
        let content = fs::read(&version.version_path)?;
        // Create a snapshot of current state before restoring
        if let Ok(current) = fs::read(&version.note_path) {
            let _ = self.create_snapshot(&version.note_path, &current);
        }
        fs::write(&version.note_path, content)?;
        Ok(())
    }

    pub fn delete_version(&self, version: &VersionInfo) -> io::Result<()> {
        if version.version_path.exists() {
            fs::remove_file(&version.version_path)?;
        }
        Ok(())
    }

    pub fn cleanup_preset_keep_count(&self, note_path: &Path, keep_count: usize) -> io::Result<usize> {
        let mut versions = self.list_versions(note_path)?;
        let mut deleted_count = 0;

        if versions.len() > keep_count {
            let to_delete = versions.split_off(keep_count);
            for v in to_delete {
                if fs::remove_file(&v.version_path).is_ok() {
                    deleted_count += 1;
                }
            }
        }
        Ok(deleted_count)
    }

    pub fn cleanup_preset_keep_days(&self, note_path: &Path, days: u64) -> io::Result<usize> {
        let versions = self.list_versions(note_path)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let max_age_sec = days * 86400;
        let mut deleted_count = 0;

        for v in versions {
            if now.saturating_sub(v.timestamp_sec) > max_age_sec {
                if fs::remove_file(&v.version_path).is_ok() {
                    deleted_count += 1;
                }
            }
        }
        Ok(deleted_count)
    }

    pub fn purge_all_for_note(&self, note_path: &Path) -> io::Result<usize> {
        let versions = self.list_versions(note_path)?;
        let mut deleted_count = 0;
        for v in versions {
            if fs::remove_file(&v.version_path).is_ok() {
                deleted_count += 1;
            }
        }
        Ok(deleted_count)
    }
}

pub fn format_timestamp(timestamp_sec: u64) -> String {
    let d = UNIX_EPOCH + std::time::Duration::from_secs(timestamp_sec);
    if let Ok(datetime) = d.duration_since(UNIX_EPOCH) {
        let secs = datetime.as_secs();
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let s = secs % 60;

        // Approximate date formatting:
        let total_days = days;
        let mut year = 1970;
        let mut rem_days = total_days;

        loop {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if leap { 366 } else { 365 };
            if rem_days < days_in_year {
                break;
            }
            rem_days -= days_in_year;
            year += 1;
        }

        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let month_days = [
            31,
            if leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut month = 1;
        for &m_days in &month_days {
            if rem_days < m_days {
                break;
            }
            rem_days -= m_days;
            month += 1;
        }
        let day = rem_days + 1;

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, mins, s
        )
    } else {
        "1970-01-01 00:00:00".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioning_snapshots_and_cleanup() {
        let temp = std::env::temp_dir().join("notedog_version_test");
        let _ = fs::remove_dir_all(&temp);
        let _ = fs::create_dir_all(&temp);

        let vm = VersionManager::new(&temp);
        let note_file = temp.join("test_note.md");
        let _ = fs::write(&note_file, "Version 1 Content");

        let p1 = vm.create_snapshot(&note_file, b"Version 1 Content").unwrap();
        assert!(p1.exists());

        let p2 = vm.create_snapshot(&note_file, b"Version 2 Content").unwrap();
        assert!(p2.exists());

        let list = vm.list_versions(&note_file).unwrap();
        assert_eq!(list.len(), 2);

        // Test preset keep count (keep 1)
        let deleted = vm.cleanup_preset_keep_count(&note_file, 1).unwrap();
        assert_eq!(deleted, 1);

        let list_after = vm.list_versions(&note_file).unwrap();
        assert_eq!(list_after.len(), 1);

        let _ = fs::remove_dir_all(&temp);
    }
}
