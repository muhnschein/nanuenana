// SPDX-License-Identifier: GPL-3.0-or-later

use crate::model::{Cached, Category, Kind, Summary, Text};
use crate::text;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningRow {
    pub id: String,
    pub category: &'static str,
    pub icon: &'static str,
    pub level: u8,
    pub kind: &'static str,
    pub headline: String,
    pub sender: String,
    pub sent_ms: i64,
    pub expires_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceView {
    pub id: u32,
    pub name: String,
    pub district_name: String,
    pub state_name: String,
    pub km: u8,
    pub current: bool,
    pub lost_since_ms: i64,
    pub updated_ms: i64,
    pub failed_since_ms: i64,
    pub stale: bool,
    pub level: u8,
    pub headline: String,
    pub warnings: Vec<WarningRow>,
    pub ended: Vec<WarningRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub notice_accepted: bool,
    pub checked_ms: i64,
    pub failing: bool,
    pub replay: bool,
    pub location: &'static str,
    pub places: Vec<PlaceView>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningView {
    pub id: String,
    pub category: &'static str,
    pub icon: &'static str,
    pub level: u8,
    pub kind: &'static str,
    pub version: u32,
    pub headline: String,
    pub sender: String,
    pub sent_ms: i64,
    pub changed_ms: i64,
    pub expires_ms: i64,
    pub description: String,
    pub instruction: String,
    pub area: String,
    pub contact: String,
    pub web: String,
    pub language: String,
    pub original: bool,
    pub detailed: bool,
    pub place: String,
    pub updated_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub name: String,
    pub district: String,
    pub district_name: String,
    pub state_name: String,
    pub postcode: String,
    pub saved: bool,
    pub match_start: usize,
    pub match_length: usize,
}

const WEATHER_ICONS: [(&str, &[&str]); 7] = [
    ("storm", &["GEWITTER"]),
    ("snow", &["SCHNEE"]),
    ("ice", &["GLÄTTE", "GLATTEIS", "FROST"]),
    ("fog", &["NEBEL"]),
    ("rain", &["REGEN"]),
    ("wind", &["BÖEN", "STURM", "ORKAN", "WIND"]),
    ("heat", &["HITZE", "UV-"]),
];

pub fn icon(category: Category, headline: &str) -> &'static str {
    if category != Category::Weather {
        return category.key();
    }
    let loud = headline.to_uppercase();
    WEATHER_ICONS
        .iter()
        .find(|(_, words)| words.iter().any(|word| loud.contains(word)))
        .map_or("weather", |(name, _)| name)
}

pub fn kind_key(kind: Kind) -> &'static str {
    match kind {
        Kind::Alert => "alert",
        Kind::Update => "update",
        Kind::Cancel => "cancel",
    }
}

fn headline_of(summary: &Summary, chosen: Option<&Text>) -> String {
    chosen
        .map(|t| t.headline.clone())
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| summary.headline.clone())
}

pub fn headline(summary: &Summary, cached: Option<&Cached>, language: &str) -> String {
    headline_of(summary, text::pick(texts_of(summary, cached), language))
}

pub fn row(summary: &Summary, cached: Option<&Cached>, language: &str) -> WarningRow {
    let texts = texts_of(summary, cached);
    let chosen = text::pick(texts, language);
    let headline = headline_of(summary, chosen);
    WarningRow {
        id: summary.id.clone(),
        category: summary.category.key(),
        icon: icon(summary.category, &summary.headline),
        level: summary.level.number(),
        kind: kind_key(summary.kind),
        headline,
        sender: sender_of(texts, chosen),
        sent_ms: summary.sent_ms,
        expires_ms: summary.expires_ms,
    }
}

fn texts_of<'a>(summary: &Summary, cached: Option<&'a Cached>) -> &'a [Text] {
    cached
        .filter(|c| c.version == summary.version)
        .and_then(|c| c.detail.as_ref())
        .map_or(&[], |d| d.texts.as_slice())
}

fn sender_of(texts: &[Text], chosen: Option<&Text>) -> String {
    chosen
        .map(|t| t.sender.clone())
        .filter(|s| !s.is_empty())
        .or_else(|| text::original(texts).map(|t| t.sender.clone()))
        .unwrap_or_default()
}

pub fn warning(summary: &Summary, cached: Option<&Cached>, language: &str) -> WarningView {
    let texts = texts_of(summary, cached);
    let chosen = text::pick(texts, language);
    let original = text::original(texts);
    let base = row(summary, cached, language);
    let detail = cached
        .filter(|c| c.version == summary.version)
        .and_then(|c| c.detail.as_ref());
    WarningView {
        id: base.id,
        category: base.category,
        icon: base.icon,
        level: base.level,
        kind: base.kind,
        version: summary.version,
        headline: base.headline,
        sender: base.sender,
        sent_ms: summary.sent_ms,
        changed_ms: detail.map_or(0, |d| d.sent_ms),
        expires_ms: summary.expires_ms,
        description: chosen.map(|t| t.description.clone()).unwrap_or_default(),
        instruction: chosen.map(|t| t.instruction.clone()).unwrap_or_default(),
        area: chosen.map(|t| t.area.clone()).unwrap_or_default(),
        contact: chosen.map(|t| t.contact.clone()).unwrap_or_default(),
        web: chosen.map(|t| t.web.clone()).unwrap_or_default(),
        language: chosen.map(|t| t.language.clone()).unwrap_or_default(),
        original: match (chosen, original) {
            (Some(c), Some(o)) => std::ptr::eq(c, o),
            _ => true,
        },
        detailed: chosen.is_some(),
        place: String::new(),
        updated_ms: 0,
    }
}

pub fn sort_rows(rows: &mut [WarningRow]) {
    rows.sort_by(|a, b| {
        b.level
            .cmp(&a.level)
            .then_with(|| b.sent_ms.cmp(&a.sent_ms))
            .then_with(|| a.id.cmp(&b.id))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Detail, Level};

    fn summary() -> Summary {
        Summary {
            id: "mow.a".to_owned(),
            version: 2,
            category: Category::Civil,
            level: Level::Danger,
            kind: Kind::Update,
            headline: "Kurz".to_owned(),
            sent_ms: 100,
            expires_ms: 200,
        }
    }

    fn cached(version: u32) -> Cached {
        Cached {
            version,
            detail: Some(Detail {
                sent_ms: 150,
                texts: vec![
                    Text {
                        language: "de".to_owned(),
                        headline: "Lang".to_owned(),
                        description: "Beschreibung".to_owned(),
                        instruction: "Handeln".to_owned(),
                        sender: "Leitstelle".to_owned(),
                        area: "Moosach".to_owned(),
                        contact: "Telefon".to_owned(),
                        web: "https://example.org".to_owned(),
                    },
                    Text {
                        language: "en".to_owned(),
                        headline: "Long".to_owned(),
                        description: "Description".to_owned(),
                        ..Text::default()
                    },
                ],
            }),
            area: None,
        }
    }

    #[test]
    fn row_without_detail_uses_the_summary() {
        let r = row(&summary(), None, "de");
        assert_eq!(
            r,
            WarningRow {
                id: "mow.a".to_owned(),
                category: "civil",
                icon: "civil",
                level: 3,
                kind: "update",
                headline: "Kurz".to_owned(),
                sender: String::new(),
                sent_ms: 100,
                expires_ms: 200,
            }
        );
    }

    #[test]
    fn row_takes_headline_and_sender_from_the_detail() {
        let c = cached(2);
        let german = row(&summary(), Some(&c), "de");
        assert_eq!(
            (german.headline.as_str(), german.sender.as_str()),
            ("Lang", "Leitstelle")
        );
        let english = row(&summary(), Some(&c), "en");
        assert_eq!(
            (english.headline.as_str(), english.sender.as_str()),
            ("Long", "Leitstelle")
        );
    }

    #[test]
    fn detail_of_another_version_is_ignored() {
        let c = cached(1);
        let r = row(&summary(), Some(&c), "de");
        assert_eq!(r.headline, "Kurz");
        let w = warning(&summary(), Some(&c), "de");
        assert!(!w.detailed);
        assert_eq!(w.changed_ms, 0);
        assert_eq!(w.description, "");
    }

    #[test]
    fn warning_view_in_the_wanted_language() {
        let c = cached(2);
        let german = warning(&summary(), Some(&c), "de_DE");
        assert!(german.detailed);
        assert!(german.original);
        assert_eq!(german.language, "de");
        assert_eq!(german.description, "Beschreibung");
        assert_eq!(german.instruction, "Handeln");
        assert_eq!(german.area, "Moosach");
        assert_eq!(german.contact, "Telefon");
        assert_eq!(german.web, "https://example.org");
        assert_eq!(german.changed_ms, 150);
        assert_eq!((german.version, german.sent_ms, german.expires_ms), (2, 100, 200));
        let english = warning(&summary(), Some(&c), "en");
        assert!(!english.original);
        assert_eq!(english.language, "en");
        assert_eq!(english.description, "Description");
        assert_eq!(english.instruction, "");
        let finnish = warning(&summary(), Some(&c), "fi");
        assert!(finnish.original);
        assert_eq!(finnish.language, "de");
    }

    #[test]
    fn warning_view_without_detail_says_so() {
        let w = warning(&summary(), None, "de");
        assert!(!w.detailed);
        assert!(w.original);
        assert_eq!(w.headline, "Kurz");
        assert_eq!(w.kind, "update");
    }

    #[test]
    fn weather_icon_follows_the_kind_of_weather_in_the_issuers_headline() {
        let cases = [
            ("Amtliche WARNUNG vor GEWITTER", "storm"),
            ("Amtliche WARNUNG vor SCHNEEFALL", "snow"),
            ("Amtliche WARNUNG vor SCHNEEVERWEHUNG", "snow"),
            ("Amtliche WARNUNG vor GLÄTTE", "ice"),
            ("Amtliche UNWETTERWARNUNG vor GLATTEIS", "ice"),
            ("Amtliche WARNUNG vor STRENGEM FROST", "ice"),
            ("Amtliche WARNUNG vor NEBEL", "fog"),
            ("Amtliche WARNUNG vor DAUERREGEN", "rain"),
            ("Amtliche WARNUNG vor BÖEN", "wind"),
            ("Amtliche WARNUNG vor STURM", "wind"),
            ("Amtliche WARNUNG vor ORKAN", "wind"),
            ("Amtliche WARNUNG vor STARKWIND", "wind"),
            ("Amtliche WARNUNG vor HITZE", "heat"),
            ("Amtliche WARNUNG vor erhöhter UV-Intensität", "heat"),
            ("Amtliche Warnung vor Glätte", "ice"),
            ("Amtliche WARNUNG vor TAUWETTER", "weather"),
            ("", "weather"),
        ];
        for (headline, wanted) in cases {
            assert_eq!(icon(Category::Weather, headline), wanted, "{headline}");
        }
    }

    #[test]
    fn of_two_kinds_of_weather_in_one_headline_the_more_telling_icon_wins() {
        let cases = [
            (
                "Amtliche UNWETTERWARNUNG vor SCHWEREM GEWITTER mit ORKANBÖEN und STARKREGEN",
                "storm",
            ),
            ("Amtliche WARNUNG vor SCHNEEFALL und WINDBÖEN", "snow"),
            ("Amtliche WARNUNG vor GLATTEIS durch gefrierenden REGEN", "ice"),
            ("Amtliche WARNUNG vor NEBEL und REGEN", "fog"),
            ("Amtliche WARNUNG vor STARKREGEN und STURMBÖEN", "rain"),
            ("Amtliche WARNUNG vor WIND und HITZE", "wind"),
        ];
        for (headline, wanted) in cases {
            assert_eq!(icon(Category::Weather, headline), wanted, "{headline}");
        }
    }

    #[test]
    fn other_categories_keep_their_own_icon_whatever_the_headline_says() {
        assert_eq!(icon(Category::Civil, "Sturmflut"), "civil");
        assert_eq!(icon(Category::Police, "GEWITTER"), "police");
        assert_eq!(icon(Category::Flood, "Hochwasser nach DAUERREGEN"), "flood");
    }

    #[test]
    fn every_icon_name_has_its_picture_in_the_app() {
        let icons = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../qml/icons");
        let categories = [
            Category::Civil,
            Category::Police,
            Category::Weather,
            Category::Flood,
        ];
        let names = categories
            .into_iter()
            .map(Category::key)
            .chain(WEATHER_ICONS.iter().map(|(name, _)| *name));
        for name in names {
            let picture = icons.join(format!("cat-{name}.svg"));
            assert!(picture.is_file(), "{}", picture.display());
        }
    }

    #[test]
    fn rows_and_warning_views_carry_the_icon_of_the_summary_headline() {
        let snow = Summary {
            category: Category::Weather,
            headline: "Amtliche WARNUNG vor SCHNEEFALL".to_owned(),
            ..summary()
        };
        let c = cached(2);
        assert_eq!(row(&snow, None, "de").icon, "snow");
        assert_eq!(row(&snow, Some(&c), "en").icon, "snow");
        assert_eq!(warning(&snow, Some(&c), "en").icon, "snow");
        assert_eq!(warning(&summary(), None, "de").icon, "civil");
    }

    #[test]
    fn kinds_have_keys() {
        assert_eq!(
            [Kind::Alert, Kind::Update, Kind::Cancel].map(kind_key),
            ["alert", "update", "cancel"]
        );
    }

    #[test]
    fn rows_sort_by_level_then_newest_then_id() {
        let make = |id: &str, level: u8, sent_ms: i64| WarningRow {
            id: id.to_owned(),
            category: "civil",
            icon: "civil",
            level,
            kind: "alert",
            headline: String::new(),
            sender: String::new(),
            sent_ms,
            expires_ms: 0,
        };
        let mut rows = vec![
            make("d", 1, 50),
            make("b", 3, 10),
            make("c", 3, 20),
            make("a", 3, 20),
            make("e", 4, 1),
        ];
        sort_rows(&mut rows);
        let order: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(order, vec!["e", "a", "c", "b", "d"]);
    }
}
