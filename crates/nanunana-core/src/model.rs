// SPDX-License-Identifier: GPL-3.0-or-later

use crate::geo::Shape;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Information,
    Warning,
    Danger,
    Extreme,
}

impl Level {
    pub fn number(self) -> u8 {
        match self {
            Level::Information => 1,
            Level::Warning => 2,
            Level::Danger => 3,
            Level::Extreme => 4,
        }
    }

    pub fn from_severity(severity: &str) -> Level {
        match severity.trim().to_ascii_lowercase().as_str() {
            "extreme" => Level::Extreme,
            "severe" => Level::Danger,
            "moderate" => Level::Warning,
            _ => Level::Information,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Civil,
    Police,
    Weather,
    Flood,
}

impl Category {
    pub fn from_provider(provider: &str, id: &str) -> Category {
        let provider = provider.trim().to_ascii_uppercase();
        let prefix = id.split('.').next().unwrap_or_default().to_ascii_lowercase();
        match (provider.as_str(), prefix.as_str()) {
            ("DWD", _) | (_, "dwd") => Category::Weather,
            ("LHP", _) | (_, "lhp") => Category::Flood,
            ("POLICE", _) | (_, "pol") => Category::Police,
            _ => Category::Civil,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Category::Civil => "civil",
            Category::Police => "police",
            Category::Weather => "weather",
            Category::Flood => "flood",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Alert,
    Update,
    Cancel,
}

impl Kind {
    pub fn from_message_type(message_type: &str) -> Kind {
        match message_type.trim().to_ascii_lowercase().as_str() {
            "cancel" => Kind::Cancel,
            "update" => Kind::Update,
            _ => Kind::Alert,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub version: u32,
    pub category: Category,
    pub level: Level,
    pub kind: Kind,
    pub headline: String,
    pub sent_ms: i64,
    pub expires_ms: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub language: String,
    pub headline: String,
    pub description: String,
    pub instruction: String,
    pub sender: String,
    pub area: String,
    pub contact: String,
    pub web: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detail {
    pub sent_ms: i64,
    pub texts: Vec<Text>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Cached {
    pub version: u32,
    pub detail: Option<Detail>,
    pub area: Option<Shape>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_maps_to_four_levels() {
        assert_eq!(Level::from_severity("Extreme"), Level::Extreme);
        assert_eq!(Level::from_severity("severe"), Level::Danger);
        assert_eq!(Level::from_severity(" Moderate "), Level::Warning);
        assert_eq!(Level::from_severity("Minor"), Level::Information);
        assert_eq!(Level::from_severity("Unknown"), Level::Information);
        assert_eq!(Level::from_severity(""), Level::Information);
    }

    #[test]
    fn levels_order_and_number() {
        assert!(Level::Extreme > Level::Danger);
        assert!(Level::Danger > Level::Warning);
        assert!(Level::Warning > Level::Information);
        let numbers: Vec<u8> = [Level::Information, Level::Warning, Level::Danger, Level::Extreme]
            .iter()
            .map(|l| l.number())
            .collect();
        assert_eq!(numbers, vec![1, 2, 3, 4]);
    }

    #[test]
    fn category_from_provider_or_id_prefix() {
        assert_eq!(Category::from_provider("MOWAS", "mow.DE-1"), Category::Civil);
        assert_eq!(Category::from_provider("KATWARN", "kat.1"), Category::Civil);
        assert_eq!(Category::from_provider("BIWAPP", "biw.1"), Category::Civil);
        assert_eq!(Category::from_provider("DWD", "x"), Category::Weather);
        assert_eq!(Category::from_provider("", "dwd.2.49"), Category::Weather);
        assert_eq!(Category::from_provider("lhp", "x"), Category::Flood);
        assert_eq!(Category::from_provider("", "lhp.A"), Category::Flood);
        assert_eq!(Category::from_provider("POLICE", "x"), Category::Police);
        assert_eq!(Category::from_provider("", "pol.1"), Category::Police);
        assert_eq!(Category::from_provider("", ""), Category::Civil);
    }

    #[test]
    fn category_keys_are_distinct() {
        let keys = [
            Category::Civil,
            Category::Police,
            Category::Weather,
            Category::Flood,
        ]
        .map(Category::key);
        assert_eq!(keys, ["civil", "police", "weather", "flood"]);
    }

    #[test]
    fn message_type_maps_to_kind() {
        assert_eq!(Kind::from_message_type("Cancel"), Kind::Cancel);
        assert_eq!(Kind::from_message_type("UPDATE"), Kind::Update);
        assert_eq!(Kind::from_message_type("Alert"), Kind::Alert);
        assert_eq!(Kind::from_message_type("?"), Kind::Alert);
    }
}
