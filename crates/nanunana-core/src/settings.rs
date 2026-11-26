// SPDX-License-Identifier: GPL-3.0-or-later

use crate::model::{Category, Level};
use serde::{Deserialize, Serialize};

pub const INTERVALS: &[u8] = &[5, 10, 15, 30];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Threshold {
    Information,
    Warning,
    Danger,
    Extreme,
    Never,
}

impl Threshold {
    pub fn admits(self, level: Level) -> bool {
        match self {
            Threshold::Information => true,
            Threshold::Warning => level >= Level::Warning,
            Threshold::Danger => level >= Level::Danger,
            Threshold::Extreme => level >= Level::Extreme,
            Threshold::Never => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub civil: Threshold,
    pub police: Threshold,
    pub weather: Threshold,
    pub flood: Threshold,
    pub all_clear: bool,
    pub every_update: bool,
    pub interval: u8,
    pub notify_failure: bool,
    pub language: String,
    pub location: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            civil: Threshold::Information,
            police: Threshold::Information,
            weather: Threshold::Warning,
            flood: Threshold::Warning,
            all_clear: true,
            every_update: false,
            interval: 5,
            notify_failure: true,
            language: String::new(),
            location: false,
        }
    }
}

impl Settings {
    pub fn threshold(&self, category: Category) -> Threshold {
        match category {
            Category::Civil => self.civil,
            Category::Police => self.police,
            Category::Weather => self.weather,
            Category::Flood => self.flood,
        }
    }

    pub fn alerts(&self, category: Category, level: Level) -> bool {
        self.threshold(category).admits(level)
    }

    pub fn interval_ms(&self) -> i64 {
        i64::from(self.interval) * 60_000
    }

    pub fn with(&self, key: &str, value: serde_json::Value) -> Option<Settings> {
        let mut map = match serde_json::to_value(self) {
            Ok(serde_json::Value::Object(map)) => map,
            _ => return None,
        };
        if !map.contains_key(key) {
            return None;
        }
        map.insert(key.to_owned(), value);
        let changed: Settings = serde_json::from_value(serde_json::Value::Object(map)).ok()?;
        INTERVALS.contains(&changed.interval).then_some(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_follow_the_design() {
        let s = Settings::default();
        assert_eq!(s.civil, Threshold::Information);
        assert_eq!(s.police, Threshold::Information);
        assert_eq!(s.weather, Threshold::Warning);
        assert_eq!(s.flood, Threshold::Warning);
        assert!(s.all_clear);
        assert!(!s.every_update);
        assert_eq!(s.interval, 5);
        assert_eq!(s.interval_ms(), 300_000);
        assert!(s.notify_failure);
        assert_eq!(s.language, "");
        assert!(!s.location);
    }

    #[test]
    fn thresholds_admit_their_level_and_higher() {
        let levels = [Level::Information, Level::Warning, Level::Danger, Level::Extreme];
        let admitted = |t: Threshold| levels.map(|l| t.admits(l));
        assert_eq!(admitted(Threshold::Information), [true, true, true, true]);
        assert_eq!(admitted(Threshold::Warning), [false, true, true, true]);
        assert_eq!(admitted(Threshold::Danger), [false, false, true, true]);
        assert_eq!(admitted(Threshold::Extreme), [false, false, false, true]);
        assert_eq!(admitted(Threshold::Never), [false, false, false, false]);
    }

    #[test]
    fn each_category_has_its_own_threshold() {
        let s = Settings {
            civil: Threshold::Never,
            police: Threshold::Extreme,
            weather: Threshold::Danger,
            flood: Threshold::Information,
            ..Settings::default()
        };
        assert_eq!(s.threshold(Category::Civil), Threshold::Never);
        assert_eq!(s.threshold(Category::Police), Threshold::Extreme);
        assert_eq!(s.threshold(Category::Weather), Threshold::Danger);
        assert_eq!(s.threshold(Category::Flood), Threshold::Information);
        assert!(!s.alerts(Category::Civil, Level::Extreme));
        assert!(s.alerts(Category::Police, Level::Extreme));
        assert!(!s.alerts(Category::Weather, Level::Warning));
        assert!(s.alerts(Category::Flood, Level::Information));
    }

    #[test]
    fn with_changes_one_key() {
        let s = Settings::default();
        let changed = s.with("weather", json!("danger")).unwrap();
        assert_eq!(changed.weather, Threshold::Danger);
        assert_eq!(changed.civil, s.civil);
        assert!(s.with("allClear", json!(false)).is_some_and(|c| !c.all_clear));
        assert_eq!(s.with("interval", json!(15)).unwrap().interval, 15);
        assert_eq!(s.with("language", json!("en")).unwrap().language, "en");
        assert!(s.with("location", json!(true)).unwrap().location);
    }

    #[test]
    fn with_rejects_unknown_keys_and_bad_values() {
        let s = Settings::default();
        assert!(s.with("nope", json!(1)).is_none());
        assert!(s.with("weather", json!("loud")).is_none());
        assert!(s.with("allClear", json!("yes")).is_none());
        assert!(s.with("interval", json!(7)).is_none());
        assert!(s.with("interval", json!(0)).is_none());
        assert!(s.with("interval", json!(-5)).is_none());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let s: Settings = serde_json::from_str(r#"{"weather":"never"}"#).unwrap();
        assert_eq!(s.weather, Threshold::Never);
        assert_eq!(s.interval, 5);
        assert!(s.all_clear);
    }

    #[test]
    fn keys_are_camel_case() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        for key in [
            "allClear",
            "everyUpdate",
            "notifyFailure",
            "interval",
            "language",
            "location",
            "civil",
        ] {
            assert!(value.get(key).is_some(), "{key}");
        }
    }
}
