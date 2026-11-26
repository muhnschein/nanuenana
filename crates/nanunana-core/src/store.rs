// SPDX-License-Identifier: GPL-3.0-or-later

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: impl Into<PathBuf>) -> Store {
        Store { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn load<T: DeserializeOwned + Default>(&self, name: &str) -> T {
        match fs::read(self.dir.join(name)) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
                log::warn!("{name} unreadable, starting empty: {e}");
                T::default()
            }),
            Err(_) => T::default(),
        }
    }

    pub fn save<T: Serialize>(&self, name: &str, value: &T) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
        let part = self.dir.join(format!("{name}.part"));
        fs::write(&part, bytes)?;
        fs::rename(&part, self.dir.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    type Map = BTreeMap<String, u32>;

    #[test]
    fn saved_values_load_again() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("deep/er"));
        let mut map = Map::new();
        map.insert("a".to_owned(), 7);
        store.save("m.json", &map).unwrap();
        assert_eq!(store.load::<Map>("m.json"), map);
        assert!(!store.dir().join("m.json.part").exists());
        assert!(store.dir().ends_with("deep/er"));
    }

    #[test]
    fn missing_or_broken_files_load_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        assert_eq!(store.load::<Map>("none.json"), Map::new());
        fs::write(dir.path().join("bad.json"), b"{not json").unwrap();
        assert_eq!(store.load::<Map>("bad.json"), Map::new());
    }

    #[test]
    fn save_replaces_the_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let mut map = Map::new();
        map.insert("a".to_owned(), 1);
        store.save("m.json", &map).unwrap();
        map.insert("a".to_owned(), 2);
        store.save("m.json", &map).unwrap();
        assert_eq!(store.load::<Map>("m.json")["a"], 2);
    }

    #[test]
    fn save_reports_a_folder_it_cannot_make() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file"), b"x").unwrap();
        let store = Store::new(dir.path().join("file/inside"));
        assert!(store.save("m.json", &Map::new()).is_err());
    }
}
