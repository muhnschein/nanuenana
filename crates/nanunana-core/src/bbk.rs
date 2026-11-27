// SPDX-License-Identifier: GPL-3.0-or-later

use crate::geo::{Point, Polygon, Ring, Shape};
use crate::model::{Category, Detail, Kind, Level, Summary, Text};
use crate::text::plain;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeMap;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const BASE: &str = "https://warnung.bund.de/api31";

fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawData {
    #[serde(deserialize_with = "lenient")]
    headline: String,
    #[serde(deserialize_with = "lenient")]
    provider: String,
    #[serde(deserialize_with = "lenient")]
    severity: String,
    #[serde(rename = "msgType", deserialize_with = "lenient")]
    message_type: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawPayload {
    #[serde(deserialize_with = "lenient")]
    version: u32,
    #[serde(deserialize_with = "lenient")]
    data: RawData,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawEntry {
    #[serde(deserialize_with = "lenient")]
    id: String,
    #[serde(deserialize_with = "lenient")]
    payload: RawPayload,
    #[serde(rename = "i18nTitle", deserialize_with = "lenient")]
    titles: BTreeMap<String, String>,
    #[serde(deserialize_with = "lenient")]
    sent: String,
    #[serde(deserialize_with = "lenient")]
    expires: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawPair {
    #[serde(rename = "valueName", deserialize_with = "lenient")]
    name: String,
    #[serde(deserialize_with = "lenient")]
    value: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawArea {
    #[serde(rename = "areaDesc", deserialize_with = "lenient")]
    description: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawInfo {
    #[serde(deserialize_with = "lenient")]
    language: String,
    #[serde(deserialize_with = "lenient")]
    headline: String,
    #[serde(deserialize_with = "lenient")]
    description: String,
    #[serde(deserialize_with = "lenient")]
    instruction: String,
    #[serde(rename = "senderName", deserialize_with = "lenient")]
    sender: String,
    #[serde(deserialize_with = "lenient")]
    web: String,
    #[serde(deserialize_with = "lenient")]
    contact: String,
    #[serde(deserialize_with = "lenient")]
    area: Vec<RawArea>,
    #[serde(deserialize_with = "lenient")]
    parameter: Vec<RawPair>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawWarning {
    #[serde(deserialize_with = "lenient")]
    sent: String,
    #[serde(deserialize_with = "lenient")]
    info: Vec<RawInfo>,
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 200
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

pub fn valid_district(district: &str) -> bool {
    district.len() == 5 && district.chars().all(|c| c.is_ascii_digit())
}

pub fn dashboard_url(district: &str) -> Option<String> {
    valid_district(district).then(|| format!("{BASE}/dashboard/{district}0000000.json"))
}

pub fn detail_url(id: &str) -> Option<String> {
    valid_id(id).then(|| format!("{BASE}/warnings/{id}.json"))
}

pub fn area_url(id: &str) -> Option<String> {
    valid_id(id).then(|| format!("{BASE}/warnings/{id}.geojson"))
}

pub fn parse_time_ms(text: &str) -> i64 {
    OffsetDateTime::parse(text.trim(), &Rfc3339)
        .map(|t| i64::try_from(t.unix_timestamp_nanos() / 1_000_000).unwrap_or_default())
        .unwrap_or_default()
}

fn summary(entry: RawEntry) -> Option<Summary> {
    if !valid_id(&entry.id) {
        return None;
    }
    let data = entry.payload.data;
    let headline = [
        Some(&data.headline),
        entry.titles.get("de"),
        entry.titles.values().next(),
    ]
    .into_iter()
    .flatten()
    .map(|h| plain(h))
    .find(|h| !h.is_empty())
    .unwrap_or_default();
    Some(Summary {
        category: Category::from_provider(&data.provider, &entry.id),
        level: Level::from_severity(&data.severity),
        kind: Kind::from_message_type(&data.message_type),
        id: entry.id,
        version: entry.payload.version,
        headline,
        sent_ms: parse_time_ms(&entry.sent),
        expires_ms: parse_time_ms(&entry.expires),
    })
}

pub fn parse_dashboard(bytes: &[u8]) -> Result<Vec<Summary>, serde_json::Error> {
    let entries: Vec<Value> = serde_json::from_slice(bytes)?;
    let mut found: Vec<Summary> = Vec::new();
    for entry in entries {
        let Some(parsed) = serde_json::from_value(entry).ok().and_then(summary) else {
            continue;
        };
        if !found.iter().any(|known| known.id == parsed.id) {
            found.push(parsed);
        }
    }
    Ok(found)
}

fn parameter<'a>(info: &'a RawInfo, name: &str) -> &'a str {
    info.parameter
        .iter()
        .find(|pair| pair.name == name)
        .map_or("", |pair| pair.value.as_str())
}

fn first_filled(candidates: [&str; 2]) -> String {
    candidates
        .into_iter()
        .map(plain)
        .find(|text| !text.is_empty())
        .unwrap_or_default()
}

fn text(info: &RawInfo) -> Text {
    let areas: Vec<String> = info
        .area
        .iter()
        .map(|area| plain(&area.description))
        .filter(|area| !area.is_empty())
        .collect();
    Text {
        language: info.language.trim().to_owned(),
        headline: plain(&info.headline),
        description: plain(&info.description),
        instruction: first_filled([&info.instruction, parameter(info, "instructionText")]),
        sender: first_filled([&info.sender, parameter(info, "sender_langname")]),
        area: areas.join(", "),
        contact: plain(&info.contact),
        web: info.web.trim().to_owned(),
    }
}

pub fn parse_detail(bytes: &[u8]) -> Result<Detail, serde_json::Error> {
    let value: Value = serde_json::from_slice(bytes)?;
    if !value.is_object() {
        return Err(serde::de::Error::custom("warning is not an object"));
    }
    let raw: RawWarning = serde_json::from_value(value)?;
    Ok(Detail {
        sent_ms: parse_time_ms(&raw.sent),
        texts: raw.info.iter().map(text).collect(),
    })
}

fn ring(value: &Value) -> Ring {
    let points = value.as_array().map(Vec::as_slice).unwrap_or_default();
    points
        .iter()
        .filter_map(|pair| {
            let lon = pair.get(0)?.as_f64()?;
            let lat = pair.get(1)?.as_f64()?;
            Some(Point::new(lat, lon)).filter(|p| p.valid())
        })
        .collect()
}

fn polygon(value: &Value) -> Option<Polygon> {
    let mut rings = value.as_array()?.iter().map(ring).filter(|r| r.len() >= 3);
    let outer = rings.next()?;
    Some(Polygon {
        outer,
        holes: rings.collect(),
    })
}

fn collect(geometry: &Value, out: &mut Vec<Polygon>) {
    let coordinates = &geometry["coordinates"];
    match geometry["type"].as_str().unwrap_or_default() {
        "Polygon" => out.extend(polygon(coordinates)),
        "MultiPolygon" => {
            let parts = coordinates.as_array().map(Vec::as_slice).unwrap_or_default();
            out.extend(parts.iter().filter_map(polygon));
        }
        "GeometryCollection" => {
            let parts = geometry["geometries"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            for part in parts {
                collect(part, out);
            }
        }
        _ => {}
    }
}

pub fn parse_area(bytes: &[u8]) -> Result<Shape, serde_json::Error> {
    let root: Value = serde_json::from_slice(bytes)?;
    let mut polygons = Vec::new();
    match root["type"].as_str().unwrap_or_default() {
        "FeatureCollection" => {
            let features = root["features"].as_array().map(Vec::as_slice).unwrap_or_default();
            for feature in features {
                collect(&feature["geometry"], &mut polygons);
            }
        }
        "Feature" => collect(&root["geometry"], &mut polygons),
        _ => collect(&root, &mut polygons),
    }
    Ok(Shape { polygons })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DASHBOARD: &str = include_str!("../tests/fixtures/dashboard-091620000000.json");
    const DETAIL: &str = include_str!("../tests/fixtures/warning-mow.json");
    const AREA: &str = include_str!("../tests/fixtures/warning-mow.geojson");

    #[test]
    fn ids_are_checked_before_they_reach_a_url() {
        assert!(valid_id("mow.DE-NW-BN-SE030-20201014-30-000"));
        assert!(valid_id("kat.60e96f0b62aefd198d9c0956_public_topics"));
        assert!(valid_id("dwd.2.49.0.0.276.0.DWD.PVW.1700000000000.abc"));
        for bad in ["", "a/b", "../x", ".hidden", "a b", "a?b", "a#b", "ä", "a%2f"] {
            assert!(!valid_id(bad), "{bad}");
        }
        assert!(valid_id(&"a".repeat(200)));
        assert!(!valid_id(&"a".repeat(201)));
    }

    #[test]
    fn urls_for_valid_input_only() {
        assert_eq!(
            dashboard_url("09162").as_deref(),
            Some("https://warnung.bund.de/api31/dashboard/091620000000.json")
        );
        assert!(dashboard_url("9162").is_none());
        assert!(dashboard_url("0916a").is_none());
        assert!(dashboard_url("091620").is_none());
        assert_eq!(
            detail_url("mow.A-1").as_deref(),
            Some("https://warnung.bund.de/api31/warnings/mow.A-1.json")
        );
        assert_eq!(
            area_url("mow.A-1").as_deref(),
            Some("https://warnung.bund.de/api31/warnings/mow.A-1.geojson")
        );
        assert!(detail_url("a/b").is_none());
        assert!(area_url("").is_none());
    }

    #[test]
    fn times_parse_with_offsets() {
        assert_eq!(parse_time_ms("1970-01-01T00:00:01Z"), 1000);
        assert_eq!(parse_time_ms("2021-07-10T11:57:33+02:00"), 1_625_911_053_000);
        assert_eq!(parse_time_ms(" 2021-07-10T09:57:33.250Z "), 1_625_911_053_250);
        assert_eq!(parse_time_ms(""), 0);
        assert_eq!(parse_time_ms("yesterday"), 0);
    }

    #[test]
    fn dashboard_entries_become_summaries() {
        let list = parse_dashboard(DASHBOARD.as_bytes()).unwrap();
        let ids: Vec<&str> = list.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "mow.DE-BY-M-SE089-20261126-89-001",
                "dwd.2.49.0.0.276.0.DWD.PVW.1795700000000.snow",
                "pol.DE-BY-M-20261126-0001",
                "lhp.HOCHWASSERZENTRALEN.DE.BY",
                "mow.no-title",
            ]
        );
        let fire = &list[0];
        assert_eq!(fire.version, 3);
        assert_eq!(fire.category, Category::Civil);
        assert_eq!(fire.level, Level::Danger);
        assert_eq!(fire.kind, Kind::Update);
        assert_eq!(fire.headline, "Rauchentwicklung durch Großbrand");
        assert_eq!(fire.sent_ms, parse_time_ms("2026-11-26T13:41:00+01:00"));
        assert_eq!(fire.expires_ms, 0);
        let snow = &list[1];
        assert_eq!(snow.category, Category::Weather);
        assert_eq!(snow.level, Level::Warning);
        assert_eq!(snow.kind, Kind::Alert);
        assert_eq!(snow.expires_ms, parse_time_ms("2026-11-27T06:00:00+01:00"));
        assert_eq!(snow.headline, "Amtliche WARNUNG vor SCHNEEFALL");
        let police = &list[2];
        assert_eq!(police.category, Category::Police);
        assert_eq!(police.kind, Kind::Cancel);
        assert_eq!(police.headline, "Entwarnung: Bombenfund in Schwabing");
        assert_eq!(list[3].category, Category::Flood);
        assert_eq!(list[3].level, Level::Information);
        assert_eq!(list[4].headline, "");
        assert_eq!(list[4].version, 0);
    }

    #[test]
    fn dashboard_rejects_what_is_not_a_list() {
        assert!(parse_dashboard(b"{}").is_err());
        assert!(parse_dashboard(b"<html>").is_err());
        assert_eq!(parse_dashboard(b"[]").unwrap(), Vec::new());
        assert_eq!(
            parse_dashboard(b"[1, \"x\", {\"id\": \"a/b\"}, {}]").unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn detail_keeps_one_text_per_language() {
        let detail = parse_detail(DETAIL.as_bytes()).unwrap();
        assert_eq!(detail.sent_ms, parse_time_ms("2026-11-26T14:05:00+01:00"));
        let languages: Vec<&str> = detail.texts.iter().map(|t| t.language.as_str()).collect();
        assert_eq!(languages, vec!["de", "de-LS", "en"]);
        let german = &detail.texts[0];
        assert_eq!(german.headline, "Rauchentwicklung durch Großbrand");
        assert_eq!(
            german.description,
            "In München-Moosach brennt eine Lagerhalle.\nEs kommt zu starker Rauchentwicklung."
        );
        assert_eq!(
            german.instruction,
            "Halten Sie Fenster und Türen geschlossen.\nMeiden Sie das betroffene Gebiet."
        );
        assert_eq!(german.sender, "Integrierte Leitstelle München");
        assert_eq!(german.area, "München: Moosach, Milbertshofen-Am Hart");
        assert_eq!(german.contact, "Bürgertelefon 089 2353-55555");
        assert_eq!(german.web, "https://www.muenchen.de");
        let english = &detail.texts[2];
        assert_eq!(english.sender, "Munich control centre");
        assert_eq!(english.instruction, "Keep windows and doors closed.");
    }

    #[test]
    fn detail_tolerates_gaps() {
        let detail = parse_detail(br#"{"info":[{"language":null,"area":null},{}],"sent":null}"#).unwrap();
        assert_eq!(detail.sent_ms, 0);
        assert_eq!(detail.texts, vec![Text::default(), Text::default()]);
        assert_eq!(parse_detail(b"{}").unwrap(), Detail::default());
        assert!(parse_detail(b"[]").is_err());
    }

    #[test]
    fn area_reads_polygons_with_longitude_first() {
        let shape = parse_area(AREA.as_bytes()).unwrap();
        assert_eq!(shape.polygons.len(), 2);
        assert_eq!(shape.polygons[0].outer.len(), 5);
        assert_eq!(shape.polygons[0].outer[0], Point::new(48.17, 11.49));
        assert_eq!(shape.polygons[0].holes.len(), 1);
        assert!(shape.contains(Point::new(48.18, 11.51)));
        assert!(!shape.contains(Point::new(48.185, 11.515)));
        assert!(shape.contains(Point::new(48.21, 11.56)));
        assert!(!shape.contains(Point::new(48.10, 11.50)));
    }

    #[test]
    fn area_accepts_bare_geometries_and_skips_the_rest() {
        let bare = br#"{"type":"Polygon","coordinates":[[[11.0,48.0],[11.1,48.0],[11.1,48.1],[11.0,48.1]]]}"#;
        assert_eq!(parse_area(bare).unwrap().polygons.len(), 1);
        let feature = br#"{"type":"Feature","geometry":{"type":"MultiPolygon","coordinates":[[[[11.0,48.0],[11.1,48.0],[11.1,48.1]]],[[[1,2]]]]}}"#;
        assert_eq!(parse_area(feature).unwrap().polygons.len(), 1);
        let mixed = br#"{"type":"GeometryCollection","geometries":[{"type":"Point","coordinates":[11.0,48.0]},{"type":"Polygon","coordinates":[[[11.0,48.0],[11.1,48.0],[11.1,48.1]]]}]}"#;
        assert_eq!(parse_area(mixed).unwrap().polygons.len(), 1);
        let off_globe = br#"{"type":"Polygon","coordinates":[[[11.0,148.0],[11.1,48.0],[11.1,48.1]]]}"#;
        assert!(parse_area(off_globe).unwrap().is_empty());
        assert!(parse_area(b"{}").unwrap().is_empty());
        assert!(parse_area(b"nope").is_err());
    }
}
