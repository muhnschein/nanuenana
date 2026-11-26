// SPDX-License-Identifier: GPL-3.0-or-later

use crate::model::{Category, Kind, Level, Summary};
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const FAILURE_AFTER_MS: i64 = 30 * 60_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seen {
    pub version: u32,
    pub level: Level,
    pub category: Category,
    pub headline: String,
    pub place: String,
    pub alerted: bool,
    pub ended: bool,
}

pub type SeenMap = BTreeMap<String, Seen>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NoticeKind {
    New,
    Raised,
    Updated,
    Ended,
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Notice {
    pub kind: NoticeKind,
    pub id: String,
    pub place: String,
    pub category: String,
    pub level: u8,
    pub headline: String,
}

impl Notice {
    pub fn failure() -> Notice {
        Notice {
            kind: NoticeKind::Failure,
            id: String::new(),
            place: String::new(),
            category: String::new(),
            level: 0,
            headline: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sighting<'a> {
    pub summary: &'a Summary,
    pub place: &'a str,
    pub quiet: bool,
}

fn merge<'a>(sightings: &[Sighting<'a>]) -> Vec<Sighting<'a>> {
    let mut merged: Vec<Sighting<'a>> = Vec::new();
    for sighting in sightings {
        match merged.iter_mut().find(|m| m.summary.id == sighting.summary.id) {
            Some(known) => known.quiet = known.quiet && sighting.quiet,
            None => merged.push(*sighting),
        }
    }
    merged
}

fn notice(kind: NoticeKind, id: &str, seen: &Seen) -> Notice {
    Notice {
        kind,
        id: id.to_owned(),
        place: seen.place.clone(),
        category: seen.category.key().to_owned(),
        level: seen.level.number(),
        headline: seen.headline.clone(),
    }
}

fn judge(old: Option<&Seen>, sighting: &Sighting, settings: &Settings) -> (Option<NoticeKind>, bool, bool) {
    let summary = sighting.summary;
    let loud = !sighting.quiet;
    let wanted = loud && settings.alerts(summary.category, summary.level);
    let active = old.filter(|o| !o.ended);
    if summary.kind == Kind::Cancel {
        let was_alerted = active.is_some_and(|o| o.alerted);
        let tell = was_alerted && loud && settings.all_clear;
        return (tell.then_some(NoticeKind::Ended), false, true);
    }
    let Some(old) = active else {
        return (wanted.then_some(NoticeKind::New), wanted, false);
    };
    if wanted && summary.level > old.level {
        return (Some(NoticeKind::Raised), true, false);
    }
    if wanted && settings.every_update && summary.version != old.version {
        return (Some(NoticeKind::Updated), true, false);
    }
    (None, old.alerted, false)
}

pub fn decide(
    seen: &mut SeenMap,
    sightings: &[Sighting],
    complete: bool,
    settings: &Settings,
) -> Vec<Notice> {
    let merged = merge(sightings);
    let mut notices = Vec::new();
    for sighting in &merged {
        let summary = sighting.summary;
        let (kind, alerted, ended) = judge(seen.get(&summary.id), sighting, settings);
        let now = Seen {
            version: summary.version,
            level: summary.level,
            category: summary.category,
            headline: summary.headline.clone(),
            place: sighting.place.to_owned(),
            alerted,
            ended,
        };
        if let Some(kind) = kind {
            notices.push(notice(kind, &summary.id, &now));
        }
        seen.insert(summary.id.clone(), now);
    }
    if complete {
        let gone: Vec<String> = seen
            .keys()
            .filter(|id| !merged.iter().any(|m| &m.summary.id == *id))
            .cloned()
            .collect();
        for id in gone {
            let Some(old) = seen.remove(&id) else { continue };
            if old.alerted && !old.ended && settings.all_clear {
                notices.push(notice(NoticeKind::Ended, &id, &old));
            }
        }
    }
    notices
}

pub fn failure_due(failing_since_ms: i64, now_ms: i64, notified: bool, settings: &Settings) -> bool {
    settings.notify_failure
        && !notified
        && failing_since_ms > 0
        && now_ms - failing_since_ms >= FAILURE_AFTER_MS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Threshold;

    fn warning(id: &str, level: Level, version: u32) -> Summary {
        Summary {
            id: id.to_owned(),
            version,
            category: Category::Civil,
            level,
            kind: Kind::Alert,
            headline: format!("headline {id}"),
            sent_ms: 1,
            expires_ms: 0,
        }
    }

    fn cancel(id: &str) -> Summary {
        Summary {
            kind: Kind::Cancel,
            ..warning(id, Level::Information, 9)
        }
    }

    fn loud<'a>(summary: &'a Summary, place: &'a str) -> Sighting<'a> {
        Sighting {
            summary,
            place,
            quiet: false,
        }
    }

    fn quiet<'a>(summary: &'a Summary, place: &'a str) -> Sighting<'a> {
        Sighting {
            summary,
            place,
            quiet: true,
        }
    }

    fn kinds(notices: &[Notice]) -> Vec<NoticeKind> {
        notices.iter().map(|n| n.kind).collect()
    }

    #[test]
    fn a_new_warning_alerts_once() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let w = warning("a", Level::Danger, 1);
        let first = decide(&mut seen, &[loud(&w, "München")], true, &settings);
        assert_eq!(
            first,
            vec![Notice {
                kind: NoticeKind::New,
                id: "a".to_owned(),
                place: "München".to_owned(),
                category: "civil".to_owned(),
                level: 3,
                headline: "headline a".to_owned(),
            }]
        );
        assert!(seen["a"].alerted);
        let second = decide(&mut seen, &[loud(&w, "München")], true, &settings);
        assert_eq!(second, vec![]);
    }

    #[test]
    fn below_threshold_stays_silent_until_it_rises() {
        let mut seen = SeenMap::new();
        let settings = Settings {
            civil: Threshold::Danger,
            ..Settings::default()
        };
        let low = warning("a", Level::Warning, 1);
        assert_eq!(decide(&mut seen, &[loud(&low, "X")], true, &settings), vec![]);
        assert!(!seen["a"].alerted);
        let high = warning("a", Level::Danger, 2);
        let raised = decide(&mut seen, &[loud(&high, "X")], true, &settings);
        assert_eq!(kinds(&raised), vec![NoticeKind::Raised]);
        assert_eq!(raised[0].level, 3);
        assert!(seen["a"].alerted);
    }

    #[test]
    fn never_threshold_silences_a_category() {
        let mut seen = SeenMap::new();
        let settings = Settings {
            civil: Threshold::Never,
            ..Settings::default()
        };
        let w = warning("a", Level::Extreme, 1);
        assert_eq!(decide(&mut seen, &[loud(&w, "X")], true, &settings), vec![]);
        assert_eq!(decide(&mut seen, &[], true, &settings), vec![]);
        assert!(seen.is_empty());
    }

    #[test]
    fn a_changed_version_alerts_only_when_asked() {
        let mut seen = SeenMap::new();
        let mut settings = Settings::default();
        let v1 = warning("a", Level::Danger, 1);
        let v2 = warning("a", Level::Danger, 2);
        decide(&mut seen, &[loud(&v1, "X")], true, &settings);
        assert_eq!(decide(&mut seen, &[loud(&v2, "X")], true, &settings), vec![]);
        assert_eq!(seen["a"].version, 2);
        assert!(seen["a"].alerted);
        settings.every_update = true;
        let v3 = warning("a", Level::Danger, 3);
        assert_eq!(
            kinds(&decide(&mut seen, &[loud(&v3, "X")], true, &settings)),
            vec![NoticeKind::Updated]
        );
        assert_eq!(decide(&mut seen, &[loud(&v3, "X")], true, &settings), vec![]);
    }

    #[test]
    fn a_falling_level_is_silent_and_a_later_rise_alerts() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let high = warning("a", Level::Danger, 1);
        let low = warning("a", Level::Warning, 2);
        decide(&mut seen, &[loud(&high, "X")], true, &settings);
        assert_eq!(decide(&mut seen, &[loud(&low, "X")], true, &settings), vec![]);
        assert_eq!(seen["a"].level, Level::Warning);
        let again = warning("a", Level::Danger, 3);
        assert_eq!(
            kinds(&decide(&mut seen, &[loud(&again, "X")], true, &settings)),
            vec![NoticeKind::Raised]
        );
    }

    #[test]
    fn cancel_tells_all_clear_once_for_alerted_warnings() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let w = warning("a", Level::Danger, 1);
        decide(&mut seen, &[loud(&w, "München")], true, &settings);
        let c = cancel("a");
        let ended = decide(&mut seen, &[loud(&c, "München")], true, &settings);
        assert_eq!(kinds(&ended), vec![NoticeKind::Ended]);
        assert_eq!(ended[0].place, "München");
        assert!(seen["a"].ended);
        assert_eq!(decide(&mut seen, &[loud(&c, "München")], true, &settings), vec![]);
        assert_eq!(decide(&mut seen, &[], true, &settings), vec![]);
        assert!(seen.is_empty());
    }

    #[test]
    fn cancel_is_silent_when_never_alerted_or_switched_off() {
        let settings = Settings::default();
        let c = cancel("a");
        let mut unseen = SeenMap::new();
        assert_eq!(decide(&mut unseen, &[loud(&c, "X")], true, &settings), vec![]);
        assert!(unseen["a"].ended);
        assert!(!unseen["a"].alerted);

        let no_all_clear = Settings {
            all_clear: false,
            ..Settings::default()
        };
        let mut seen = SeenMap::new();
        let w = warning("a", Level::Danger, 1);
        decide(&mut seen, &[loud(&w, "X")], true, &no_all_clear);
        assert_eq!(decide(&mut seen, &[loud(&c, "X")], true, &no_all_clear), vec![]);
        assert!(seen["a"].ended);
    }

    #[test]
    fn a_warning_reissued_after_cancel_is_new_again() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let w = warning("a", Level::Danger, 1);
        let c = cancel("a");
        decide(&mut seen, &[loud(&w, "X")], true, &settings);
        decide(&mut seen, &[loud(&c, "X")], true, &settings);
        let again = warning("a", Level::Information, 3);
        assert_eq!(
            kinds(&decide(&mut seen, &[loud(&again, "X")], true, &settings)),
            vec![NoticeKind::New]
        );
    }

    #[test]
    fn a_vanished_warning_ends_only_after_a_complete_check() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let w = warning("a", Level::Danger, 1);
        decide(&mut seen, &[loud(&w, "Köln")], true, &settings);
        assert_eq!(decide(&mut seen, &[], false, &settings), vec![]);
        assert!(seen.contains_key("a"));
        let ended = decide(&mut seen, &[], true, &settings);
        assert_eq!(kinds(&ended), vec![NoticeKind::Ended]);
        assert_eq!(ended[0].headline, "headline a");
        assert_eq!(ended[0].place, "Köln");
        assert_eq!(ended[0].level, 3);
        assert!(seen.is_empty());
    }

    #[test]
    fn a_vanished_warning_is_silent_without_all_clear_or_alert() {
        let w = warning("a", Level::Danger, 1);
        let no_all_clear = Settings {
            all_clear: false,
            ..Settings::default()
        };
        let mut seen = SeenMap::new();
        decide(&mut seen, &[loud(&w, "X")], true, &no_all_clear);
        assert_eq!(decide(&mut seen, &[], true, &no_all_clear), vec![]);
        assert!(seen.is_empty());

        let settings = Settings::default();
        let mut primed = SeenMap::new();
        decide(&mut primed, &[quiet(&w, "X")], true, &settings);
        assert_eq!(decide(&mut primed, &[], true, &settings), vec![]);
        assert!(primed.is_empty());
    }

    #[test]
    fn quiet_sightings_are_recorded_without_alert() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let w = warning("a", Level::Extreme, 1);
        assert_eq!(decide(&mut seen, &[quiet(&w, "X")], true, &settings), vec![]);
        assert!(!seen["a"].alerted);
        assert_eq!(decide(&mut seen, &[loud(&w, "X")], true, &settings), vec![]);
        let c = cancel("a");
        assert_eq!(decide(&mut seen, &[loud(&c, "X")], true, &settings), vec![]);
    }

    #[test]
    fn one_warning_at_two_places_alerts_once_for_the_first() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let w = warning("a", Level::Danger, 1);
        let notices = decide(
            &mut seen,
            &[loud(&w, "München"), loud(&w, "Augsburg")],
            true,
            &settings,
        );
        assert_eq!(kinds(&notices), vec![NoticeKind::New]);
        assert_eq!(notices[0].place, "München");
    }

    #[test]
    fn a_warning_is_quiet_only_if_every_place_is() {
        let settings = Settings::default();
        let w = warning("a", Level::Danger, 1);
        let mut mixed = SeenMap::new();
        let notices = decide(&mut mixed, &[quiet(&w, "New"), loud(&w, "Old")], true, &settings);
        assert_eq!(kinds(&notices), vec![NoticeKind::New]);
        let mut reversed = SeenMap::new();
        let notices = decide(
            &mut reversed,
            &[loud(&w, "Old"), quiet(&w, "New")],
            true,
            &settings,
        );
        assert_eq!(kinds(&notices), vec![NoticeKind::New]);
        let mut all_quiet = SeenMap::new();
        let notices = decide(
            &mut all_quiet,
            &[quiet(&w, "New"), quiet(&w, "Newer")],
            true,
            &settings,
        );
        assert_eq!(notices, vec![]);
    }

    #[test]
    fn several_warnings_are_judged_apart() {
        let mut seen = SeenMap::new();
        let settings = Settings::default();
        let a = warning("a", Level::Danger, 1);
        let b = warning("b", Level::Information, 1);
        let weather = Summary {
            category: Category::Weather,
            ..warning("c", Level::Information, 1)
        };
        let notices = decide(
            &mut seen,
            &[loud(&a, "X"), loud(&b, "X"), loud(&weather, "X")],
            true,
            &settings,
        );
        let ids: Vec<&str> = notices.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b"]);
        assert_eq!(seen.len(), 3);
        let after = decide(&mut seen, &[loud(&b, "X"), loud(&weather, "X")], true, &settings);
        assert_eq!(kinds(&after), vec![NoticeKind::Ended]);
        assert_eq!(after[0].id, "a");
    }

    #[test]
    fn failure_notice_after_thirty_minutes_once() {
        let settings = Settings::default();
        let start = 1_000_000;
        assert!(!failure_due(0, start + FAILURE_AFTER_MS, false, &settings));
        assert!(!failure_due(
            start,
            start + FAILURE_AFTER_MS - 1,
            false,
            &settings
        ));
        assert!(failure_due(start, start + FAILURE_AFTER_MS, false, &settings));
        assert!(failure_due(
            start,
            start + 10 * FAILURE_AFTER_MS,
            false,
            &settings
        ));
        assert!(!failure_due(start, start + FAILURE_AFTER_MS, true, &settings));
        let off = Settings {
            notify_failure: false,
            ..Settings::default()
        };
        assert!(!failure_due(start, start + FAILURE_AFTER_MS, false, &off));
        assert_eq!(FAILURE_AFTER_MS, 1_800_000);
    }

    #[test]
    fn failure_notice_has_no_warning_fields() {
        let n = Notice::failure();
        assert_eq!(n.kind, NoticeKind::Failure);
        assert_eq!((n.id.as_str(), n.level), ("", 0));
        let json = serde_json::to_value(&n).unwrap();
        assert_eq!(json["kind"], "failure");
    }
}
