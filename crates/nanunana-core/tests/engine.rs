// SPDX-License-Identifier: GPL-3.0-or-later

use nanunana_core::alerts::{NoticeKind, FAILURE_AFTER_MS};
use nanunana_core::engine::{AddError, Engine, LOCATION_ID};
use nanunana_core::net::{Fetch, FetchError};
use nanunana_core::store::Store;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

const DASHBOARD: &str = include_str!("fixtures/dashboard-091620000000.json");
const DETAIL: &str = include_str!("fixtures/warning-mow.json");
const AREA: &str = include_str!("fixtures/warning-mow.geojson");
const FIRE: &str = "mow.DE-BY-M-SE089-20261126-89-001";
const SNOW: &str = "dwd.2.49.0.0.276.0.DWD.PVW.1795700000000.snow";
const BASE: &str = "https://warnung.bund.de/api31";
const MINUTE: i64 = 60_000;
const T0: i64 = 1_795_700_000_000;

#[derive(Default)]
struct Net {
    answers: BTreeMap<String, String>,
    down: bool,
    asked: Vec<String>,
}

#[derive(Clone, Default)]
struct FakeNet(Arc<Mutex<Net>>);

impl FakeNet {
    fn answer(&self, path: &str, body: &str) {
        self.0
            .lock()
            .unwrap()
            .answers
            .insert(format!("{BASE}{path}"), body.to_owned());
    }

    fn forget(&self, path: &str) {
        self.0.lock().unwrap().answers.remove(&format!("{BASE}{path}"));
    }

    fn down(&self, down: bool) {
        self.0.lock().unwrap().down = down;
    }

    fn asked(&self) -> Vec<String> {
        self.0.lock().unwrap().asked.clone()
    }

    fn count(&self, part: &str) -> usize {
        self.asked().iter().filter(|url| url.contains(part)).count()
    }

    fn munich(&self) {
        self.answer("/dashboard/091620000000.json", DASHBOARD);
        self.answer(&format!("/warnings/{FIRE}.json"), DETAIL);
        self.answer(&format!("/warnings/{FIRE}.geojson"), AREA);
    }
}

impl Fetch for FakeNet {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let mut net = self.0.lock().unwrap();
        net.asked.push(url.to_owned());
        if net.down {
            return Err(FetchError::Transport("offline".to_owned()));
        }
        match net.answers.get(url) {
            Some(body) => Ok(body.clone().into_bytes()),
            None if url.contains("/dashboard/") => Ok(b"[]".to_vec()),
            None => Err(FetchError::Status(404)),
        }
    }
}

struct Bench {
    net: FakeNet,
    dir: tempfile::TempDir,
    engine: Engine,
}

fn open(dir: &tempfile::TempDir, net: &FakeNet) -> Engine {
    Engine::open(Store::new(dir.path()), Box::new(net.clone()))
}

fn bench() -> Bench {
    let net = FakeNet::default();
    let dir = tempfile::tempdir().unwrap();
    let engine = open(&dir, &net);
    engine.accept_notice();
    Bench { net, dir, engine }
}

fn kinds(notices: &[nanunana_core::alerts::Notice]) -> Vec<NoticeKind> {
    notices.iter().map(|n| n.kind).collect()
}

fn ids(rows: &[nanunana_core::view::WarningRow]) -> Vec<&str> {
    rows.iter().map(|r| r.id.as_str()).collect()
}

#[test]
fn nothing_is_fetched_before_the_notice_is_accepted() {
    let net = FakeNet::default();
    let dir = tempfile::tempdir().unwrap();
    let engine = open(&dir, &net);
    assert!(!engine.notice_accepted());
    engine.add_place("München", "09162", 0).unwrap();
    assert_eq!(engine.refresh(T0), vec![]);
    assert!(net.asked().is_empty());
    assert!(!engine.view(T0).notice_accepted);
    engine.accept_notice();
    assert!(engine.notice_accepted());
    engine.refresh(T0);
    assert_eq!(net.asked(), vec![format!("{BASE}/dashboard/091620000000.json")]);
    assert!(open(&dir, &net).notice_accepted());
}

#[test]
fn a_whole_district_place_asks_for_its_district_only() {
    let b = bench();
    b.net.munich();
    let id = b.engine.add_place("München", "09162", 0).unwrap();
    assert_eq!(b.engine.refresh(T0), vec![]);
    assert_eq!(b.net.count("/dashboard/"), 1);
    let view = b.engine.view(T0);
    assert_eq!(view.checked_ms, T0);
    assert_eq!(view.location, "off");
    assert_eq!(view.places.len(), 1);
    let place = &view.places[0];
    assert_eq!(place.id, id);
    assert_eq!(place.name, "München");
    assert_eq!(place.district_name, "Kreisfreie Stadt München");
    assert_eq!(place.state_name, "Bayern");
    assert_eq!(place.km, 0);
    assert!(!place.current);
    assert!(!place.stale);
    assert_eq!(place.updated_ms, T0);
    assert_eq!(place.failed_since_ms, 0);
    assert_eq!(place.level, 3);
    assert_eq!(place.headline, "Rauchentwicklung durch Großbrand");
    assert_eq!(
        ids(&place.warnings),
        vec![FIRE, SNOW, "lhp.HOCHWASSERZENTRALEN.DE.BY", "mow.no-title"]
    );
    assert_eq!(ids(&place.ended), vec!["pol.DE-BY-M-20261126-0001"]);
    assert_eq!(place.warnings[0].sender, "Integrierte Leitstelle München");
    assert_eq!(place.warnings[1].sender, "");
}

#[test]
fn a_radius_place_asks_neighbours_and_filters_by_distance() {
    let b = bench();
    b.net.munich();
    let near = b.engine.add_place("München", "09162", 5).unwrap();
    b.engine.refresh(T0);
    assert!(b.net.count("/dashboard/") > 1);
    assert_eq!(b.net.count("/dashboard/091620000000.json"), 1);
    assert_eq!(b.net.count("/dashboard/091840000000.json"), 1);
    assert_eq!(b.net.count("/dashboard/091790000000.json"), 1);
    assert_eq!(b.net.count("/dashboard/091780000000.json"), 0);
    assert_eq!(b.net.count("/dashboard/110000000000.json"), 0);
    let place = b.engine.view(T0).places.remove(0);
    assert_eq!(place.km, 5);
    assert!(ids(&place.warnings).contains(&FIRE));
    assert!(b.engine.set_area(near, 3));
    let closer = b.engine.view(T0).places.remove(0);
    assert!(!ids(&closer.warnings).contains(&FIRE));
    assert!(ids(&closer.warnings).contains(&SNOW));
    assert_eq!(closer.level, 2);
    assert!(b.engine.warning(FIRE).is_none());
}

#[test]
fn the_first_check_of_a_place_is_silent_and_later_news_alert() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.net.answer("/dashboard/091620000000.json", "[]");
    assert_eq!(b.engine.refresh(T0), vec![]);
    b.net.munich();
    let notices = b.engine.refresh(T0 + 5 * MINUTE);
    let told: Vec<(&str, NoticeKind, u8)> =
        notices.iter().map(|n| (n.id.as_str(), n.kind, n.level)).collect();
    assert_eq!(
        told,
        vec![
            (FIRE, NoticeKind::New, 3),
            (SNOW, NoticeKind::New, 2),
            ("mow.no-title", NoticeKind::New, 1),
        ]
    );
    assert_eq!(notices[0].place, "München");
    assert_eq!(notices[0].category, "civil");
    assert_eq!(notices[0].headline, "Rauchentwicklung durch Großbrand");
    assert_eq!(b.engine.refresh(T0 + 10 * MINUTE), vec![]);
    b.net.answer("/dashboard/091620000000.json", "[]");
    let ended = b.engine.refresh(T0 + 15 * MINUTE);
    assert_eq!(kinds(&ended), vec![NoticeKind::Ended; 3]);
}

#[test]
fn a_place_added_beside_existing_warnings_stays_silent() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("München", "09162", 0).unwrap();
    assert_eq!(b.engine.refresh(T0), vec![]);
    assert_eq!(b.engine.refresh(T0 + MINUTE), vec![]);
}

#[test]
fn details_are_fetched_once_per_version() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    b.engine.refresh(T0 + MINUTE);
    assert_eq!(b.net.count(&format!("/warnings/{FIRE}.json")), 1);
    assert_eq!(b.net.count(&format!("/warnings/{FIRE}.geojson")), 1);
    assert_eq!(b.net.count(&format!("/warnings/{SNOW}.json")), 2);
    let bumped = DASHBOARD.replace("\"version\": 3", "\"version\": 4");
    b.net.answer("/dashboard/091620000000.json", &bumped);
    b.engine.refresh(T0 + 2 * MINUTE);
    assert_eq!(b.net.count(&format!("/warnings/{FIRE}.json")), 2);
    assert_eq!(b.net.count(&format!("/warnings/{FIRE}.geojson")), 2);
}

#[test]
fn a_failed_check_keeps_old_warnings_and_marks_them_stale() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    b.net.down(true);
    assert_eq!(b.engine.refresh(T0 + 5 * MINUTE), vec![]);
    let view = b.engine.view(T0 + 5 * MINUTE);
    assert_eq!(view.checked_ms, T0);
    let place = &view.places[0];
    assert!(place.stale);
    assert_eq!(place.updated_ms, T0);
    assert_eq!(place.failed_since_ms, T0 + 5 * MINUTE);
    assert_eq!(place.warnings.len(), 4);
    assert_eq!(b.engine.refresh(T0 + 10 * MINUTE), vec![]);
    assert_eq!(
        b.engine.view(T0 + 10 * MINUTE).places[0].failed_since_ms,
        T0 + 5 * MINUTE
    );
}

#[test]
fn failure_is_told_once_after_thirty_minutes_and_resets_on_success() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    assert!(!b.engine.view(T0).failing);
    b.net.down(true);
    let start = T0 + 5 * MINUTE;
    assert_eq!(b.engine.refresh(start), vec![]);
    assert!(b.engine.view(start).failing);
    assert_eq!(b.engine.refresh(start + FAILURE_AFTER_MS - 1), vec![]);
    assert_eq!(
        kinds(&b.engine.refresh(start + FAILURE_AFTER_MS)),
        vec![NoticeKind::Failure]
    );
    assert_eq!(b.engine.refresh(start + 2 * FAILURE_AFTER_MS), vec![]);
    assert!(b.engine.view(start + 2 * FAILURE_AFTER_MS).failing);
    b.net.down(false);
    let back = start + 3 * FAILURE_AFTER_MS;
    assert_eq!(b.engine.refresh(back), vec![]);
    let view = b.engine.view(back);
    assert!(!view.places[0].stale);
    assert!(!view.failing);
    assert_eq!(view.checked_ms, back);
    b.net.down(true);
    assert_eq!(b.engine.refresh(back + MINUTE), vec![]);
    assert_eq!(
        kinds(&b.engine.refresh(back + MINUTE + FAILURE_AFTER_MS)),
        vec![NoticeKind::Failure]
    );
}

#[test]
fn failure_notice_can_be_switched_off() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    assert!(b.engine.set_setting("notifyFailure", json!(false)));
    b.net.down(true);
    assert_eq!(b.engine.refresh(T0), vec![]);
    assert_eq!(b.engine.refresh(T0 + FAILURE_AFTER_MS), vec![]);
}

#[test]
fn a_vanished_warning_is_not_ended_while_the_check_fails() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.net.answer("/dashboard/091620000000.json", "[]");
    b.engine.refresh(T0);
    b.net.munich();
    assert_eq!(b.engine.refresh(T0 + MINUTE).len(), 3);
    b.net.down(true);
    assert_eq!(b.engine.refresh(T0 + 2 * MINUTE), vec![]);
    b.net.down(false);
    assert_eq!(b.engine.refresh(T0 + 3 * MINUTE), vec![]);
}

#[test]
fn an_old_check_is_stale_even_without_failure() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    assert!(!b.engine.view(T0 + 11 * MINUTE).places[0].stale);
    assert!(b.engine.view(T0 + 11 * MINUTE + 1).places[0].stale);
    assert!(b.engine.set_setting("interval", json!(30)));
    assert!(!b.engine.view(T0 + 61 * MINUTE).places[0].stale);
    assert!(b.engine.view(T0 + 61 * MINUTE + 1).places[0].stale);
}

#[test]
fn a_place_never_checked_is_stale_not_clear() {
    let b = bench();
    b.engine.add_place("München", "09162", 5).unwrap();
    let place = b.engine.view(T0).places.remove(0);
    assert!(place.stale);
    assert_eq!(place.updated_ms, 0);
    assert_eq!(place.level, 0);
    assert!(place.warnings.is_empty());
}

#[test]
fn places_can_be_added_once_changed_and_removed() {
    let b = bench();
    let munich = b.engine.add_place("München", "09162", 5).unwrap();
    assert_eq!(
        b.engine.add_place("München", "09162", 9),
        Err(AddError::Duplicate)
    );
    assert_eq!(b.engine.add_place("Atlantis", "09162", 5), Err(AddError::Unknown));
    assert_eq!(b.engine.add_place("München", "11000", 5), Err(AddError::Unknown));
    let cologne = b.engine.add_place("Köln", "05315", 200).unwrap();
    assert_ne!(munich, cologne);
    assert_ne!(munich, LOCATION_ID);
    let places = b.engine.places();
    assert_eq!(places.len(), 2);
    assert_eq!(places[1].km, 25);
    assert!((places[0].lat - 48.14).abs() < 0.05 && (places[0].lon - 11.56).abs() < 0.05);
    assert!(b.engine.set_area(munich, 0));
    assert!(!b.engine.set_area(999, 3));
    assert_eq!(b.engine.places()[0].km, 0);
    assert!(b.engine.remove_place(munich));
    assert!(!b.engine.remove_place(munich));
    let again = b.engine.add_place("München", "09162", 1).unwrap();
    assert!(again > cologne);
    let names: Vec<String> = b.engine.view(T0).places.into_iter().map(|p| p.name).collect();
    assert_eq!(names, vec!["Köln".to_owned(), "München".to_owned()]);
}

#[test]
fn removing_a_place_stops_asking_for_its_district() {
    let b = bench();
    let munich = b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.add_place("Köln", "05315", 0).unwrap();
    b.engine.refresh(T0);
    assert_eq!(b.net.count("/dashboard/"), 2);
    b.engine.remove_place(munich);
    b.engine.refresh(T0 + MINUTE);
    assert_eq!(b.net.count("/dashboard/091620000000.json"), 1);
    assert_eq!(b.net.count("/dashboard/053150000000.json"), 2);
}

#[test]
fn state_survives_a_restart_without_new_alerts() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.net.answer("/dashboard/091620000000.json", "[]");
    b.engine.refresh(T0);
    b.net.munich();
    assert_eq!(b.engine.refresh(T0 + MINUTE).len(), 3);
    assert!(b.engine.set_setting("weather", json!("never")));
    let reopened = open(&b.dir, &b.net);
    assert_eq!(reopened.places().len(), 1);
    assert_eq!(
        reopened.settings().weather,
        nanunana_core::settings::Threshold::Never
    );
    let view = reopened.view(T0 + MINUTE);
    assert_eq!(view.places[0].warnings.len(), 4);
    assert_eq!(
        view.places[0].warnings[0].sender,
        "Integrierte Leitstelle München"
    );
    assert_eq!(reopened.refresh(T0 + 2 * MINUTE), vec![]);
    assert_eq!(b.net.count(&format!("/warnings/{FIRE}.json")), 1);
}

#[test]
fn settings_change_only_through_valid_keys() {
    let b = bench();
    assert!(!b.engine.set_setting("interval", json!(7)));
    assert!(!b.engine.set_setting("unknown", json!(true)));
    assert!(b.engine.set_setting("everyUpdate", json!(true)));
    assert!(b.engine.settings().every_update);
}

#[test]
fn every_update_alerts_on_a_new_version() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.net.answer("/dashboard/091620000000.json", "[]");
    b.engine.refresh(T0);
    b.net.munich();
    b.engine.refresh(T0 + MINUTE);
    let bumped = DASHBOARD.replace("\"version\": 3", "\"version\": 4");
    b.net.answer("/dashboard/091620000000.json", &bumped);
    assert_eq!(b.engine.refresh(T0 + 2 * MINUTE), vec![]);
    b.engine.set_setting("everyUpdate", json!(true));
    let again = bumped.replace("\"version\": 4", "\"version\": 5");
    b.net.answer("/dashboard/091620000000.json", &again);
    let notices = b.engine.refresh(T0 + 3 * MINUTE);
    assert_eq!(kinds(&notices), vec![NoticeKind::Updated]);
    assert_eq!(notices[0].id, FIRE);
}

#[test]
fn warning_text_follows_locale_and_language_setting() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    let german = b.engine.warning(FIRE).unwrap();
    assert_eq!(german.language, "de");
    assert!(german.original);
    assert!(german.detailed);
    assert_eq!(german.place, "München");
    assert_eq!(german.updated_ms, T0);
    assert_eq!(german.sender, "Integrierte Leitstelle München");
    b.engine.set_locale("en_GB");
    let english = b.engine.warning(FIRE).unwrap();
    assert_eq!(english.language, "en");
    assert!(!english.original);
    assert_eq!(english.headline, "Smoke from a major fire");
    assert_eq!(b.engine.view(T0).places[0].headline, "Smoke from a major fire");
    assert!(b.engine.set_setting("language", json!("de")));
    assert_eq!(b.engine.warning(FIRE).unwrap().language, "de");
    let undetailed = b.engine.warning(SNOW).unwrap();
    assert!(!undetailed.detailed);
    assert_eq!(undetailed.headline, "Amtliche WARNUNG vor SCHNEEFALL");
    assert!(b.engine.warning("mow.unknown").is_none());
}

#[test]
fn a_missing_detail_is_asked_for_again() {
    let b = bench();
    b.net.munich();
    b.net.forget(&format!("/warnings/{FIRE}.json"));
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    assert!(!b.engine.warning(FIRE).unwrap().detailed);
    b.net.munich();
    b.engine.refresh(T0 + MINUTE);
    assert!(b.engine.warning(FIRE).unwrap().detailed);
    assert_eq!(b.net.count(&format!("/warnings/{FIRE}.geojson")), 1);
}

#[test]
fn search_lists_places_and_marks_saved_ones() {
    let b = bench();
    b.engine.add_place("Rehlingen-Siersburg", "10044", 5).unwrap();
    let hits = b.engine.search(" rehl ");
    let names: Vec<&str> = hits.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(&names[..3], ["Rehling", "Rehlingen", "Rehlingen-Siersburg"]);
    let saved = &hits[2];
    assert!(saved.saved);
    assert!(!hits[0].saved);
    assert_eq!(saved.district, "10044");
    assert_eq!(saved.district_name, "Landkreis Saarlouis");
    assert_eq!(saved.state_name, "Saarland");
    assert_eq!(saved.postcode, "66780");
    let by_code = b.engine.search("80331");
    assert_eq!(by_code[0].name, "München");
    assert_eq!(by_code[0].postcode, "80331");
    assert!(b.engine.search("x").is_empty());
    assert!(b.engine.search("mü").len() <= 30);
}

#[test]
fn current_location_is_off_until_switched_on() {
    let b = bench();
    assert!(!b.engine.set_position(48.37, 10.90, T0));
    assert_eq!(b.engine.view(T0).location, "off");
    assert!(b.engine.set_setting("location", json!(true)));
    assert_eq!(b.engine.view(T0).location, "waiting");
    assert!(b.engine.view(T0).places.is_empty());
    assert!(b.engine.set_position(48.37, 10.90, T0));
    assert!(!b.engine.set_position(48.37, 10.90, T0 + 1));
    assert!(!b.engine.set_position(91.0, 10.90, T0 + 1));
    let view = b.engine.view(T0 + 1);
    assert_eq!(view.location, "on");
    let here = &view.places[0];
    assert_eq!(here.id, LOCATION_ID);
    assert!(here.current);
    assert_eq!(here.name, "Augsburg");
    assert_eq!(here.district_name, "Kreisfreie Stadt Augsburg");
    assert_eq!(here.km, 2);
    assert!(here.stale);
    assert_eq!(here.lost_since_ms, 0);
}

#[test]
fn current_location_is_first_and_alerts_when_entering_a_warned_area() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("Köln", "05315", 0).unwrap();
    b.engine.set_setting("location", json!(true));
    b.engine.set_position(48.37, 10.90, T0);
    assert_eq!(b.engine.refresh(T0), vec![]);
    assert!(b.net.count("/dashboard/097610000000.json") == 1);
    let names: Vec<String> = b.engine.view(T0).places.into_iter().map(|p| p.name).collect();
    assert_eq!(names, vec!["Augsburg".to_owned(), "Köln".to_owned()]);
    assert!(b.engine.set_position(48.172, 11.528, T0 + MINUTE));
    let notices = b.engine.refresh(T0 + MINUTE);
    assert!(notices.iter().any(|n| n.id == FIRE && n.kind == NoticeKind::New));
    assert_eq!(notices[0].place, "München");
    let here = b.engine.view(T0 + MINUTE).places.remove(0);
    assert_eq!(here.level, 3);
    assert!(!here.stale);
}

#[test]
fn current_location_outside_germany_shows_nothing() {
    let b = bench();
    b.engine.set_setting("location", json!(true));
    assert!(b.engine.set_position(48.8566, 2.3522, T0));
    let view = b.engine.view(T0);
    assert_eq!(view.location, "outside");
    assert!(view.places.is_empty());
    b.engine.refresh(T0);
    assert!(b.net.asked().is_empty());
}

#[test]
fn current_location_is_lost_when_the_fix_is_old() {
    let b = bench();
    b.engine.set_setting("location", json!(true));
    b.engine.set_position(48.37, 10.90, T0);
    b.engine.refresh(T0);
    assert_eq!(b.engine.view(T0 + 11 * MINUTE).places[0].lost_since_ms, 0);
    b.engine.refresh(T0 + 12 * MINUTE);
    let late = b.engine.view(T0 + 12 * MINUTE);
    assert_eq!(late.places[0].lost_since_ms, T0);
    assert_eq!(late.places[0].name, "Augsburg");
    assert!(!late.places[0].stale);
    b.engine.set_position(48.37, 10.90, T0 + 12 * MINUTE);
    assert_eq!(b.engine.view(T0 + 12 * MINUTE).places[0].lost_since_ms, 0);
}

#[test]
fn switching_location_off_forgets_the_position() {
    let b = bench();
    b.engine.set_setting("location", json!(true));
    b.engine.set_position(48.37, 10.90, T0);
    b.engine.set_setting("location", json!(false));
    assert!(b.engine.view(T0).places.is_empty());
    b.engine.set_setting("location", json!(true));
    assert_eq!(b.engine.view(T0).location, "waiting");
    let reopened = open(&b.dir, &b.net);
    assert_eq!(reopened.view(T0).location, "waiting");
}

#[test]
fn the_position_is_not_written_to_disk() {
    let b = bench();
    b.engine.set_setting("location", json!(true));
    b.engine.set_position(48.3712, 10.9034, T0);
    b.engine.refresh(T0);
    let state = std::fs::read_to_string(b.dir.path().join("state.json")).unwrap();
    assert!(!state.contains("48.37"), "{state}");
    assert!(!state.contains("10.90"), "{state}");
}

fn dashboard_without(id: &str) -> String {
    let mut list: Vec<serde_json::Value> = serde_json::from_str(DASHBOARD).unwrap();
    list.retain(|entry| entry["id"] != id);
    serde_json::to_string(&list).unwrap()
}

#[test]
fn places_are_told_apart_by_name_and_district_together() {
    let b = bench();
    b.engine.add_place("Aach", "07235", 0).unwrap();
    assert!(b.engine.add_place("Aach", "08335", 0).is_ok());
    b.engine.add_place("Aschheim", "09184", 0).unwrap();
    assert!(b.engine.add_place("Aying", "09184", 0).is_ok());
    assert_eq!(b.engine.add_place("Aying", "09184", 5), Err(AddError::Duplicate));
    b.engine.add_place("Achim", "03158", 0).unwrap();
    let saved: Vec<(String, bool)> = b
        .engine
        .search("Achim")
        .into_iter()
        .filter(|hit| hit.name == "Achim")
        .map(|hit| (hit.district, hit.saved))
        .collect();
    assert_eq!(
        saved,
        vec![("03158".to_owned(), true), ("03361".to_owned(), false)]
    );
    let neighbour = b.engine.search("Brunnthal").remove(0);
    assert_eq!((neighbour.district.as_str(), neighbour.saved), ("09184", false));
}

#[test]
fn another_setting_keeps_the_position_and_switching_location_off_forgets_it() {
    let b = bench();
    assert!(b.engine.set_setting("location", json!(true)));
    assert!(b.engine.set_position(48.37, 10.90, T0));
    assert_eq!(b.engine.view(T0).location, "on");
    assert!(b.engine.set_setting("interval", json!(15)));
    assert_eq!(b.engine.view(T0).location, "on");
    assert!(b.engine.set_setting("location", json!(false)));
    assert!(b.engine.set_setting("location", json!(true)));
    assert_eq!(b.engine.view(T0).location, "waiting");
}

#[test]
fn details_of_a_warning_that_left_are_dropped_and_asked_again_when_it_returns() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("München", "09162", 0).unwrap();
    let detail = format!("/warnings/{FIRE}.json");
    b.engine.refresh(T0);
    b.engine.refresh(T0 + MINUTE);
    assert_eq!(b.net.count(&detail), 1);
    b.net
        .answer("/dashboard/091620000000.json", &dashboard_without(FIRE));
    b.engine.refresh(T0 + 2 * MINUTE);
    let reopened = open(&b.dir, &b.net);
    b.net.munich();
    reopened.refresh(T0 + 3 * MINUTE);
    assert_eq!(b.net.count(&detail), 2);
}

#[test]
fn a_failed_check_at_the_time_of_a_good_one_still_counts_as_failed() {
    let b = bench();
    b.net.munich();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.engine.refresh(T0);
    assert!(!b.engine.view(T0).failing);
    b.net.down(true);
    b.engine.refresh(T0);
    assert!(b.engine.view(T0).failing);
}

#[test]
fn news_after_a_failed_check_still_alert() {
    let b = bench();
    b.engine.add_place("München", "09162", 0).unwrap();
    b.net.answer("/dashboard/091620000000.json", "[]");
    b.engine.refresh(T0);
    b.net.down(true);
    b.engine.refresh(T0 + MINUTE);
    b.net.down(false);
    b.net.munich();
    assert_eq!(b.engine.refresh(T0 + 2 * MINUTE).len(), 3);
}
