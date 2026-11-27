// SPDX-License-Identifier: GPL-3.0-or-later

use nanunana_core::alerts::NoticeKind;
use nanunana_core::engine::Engine;
use nanunana_core::net::{Canned, Fetch, FetchError};
use nanunana_core::source::{source, Source};
use nanunana_core::store::Store;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const T0: i64 = 1_795_700_000_000;
const DASHBOARD: &[u8] = include_bytes!("fixtures/dashboard-091620000000.json");

struct Offline;

impl Fetch for Offline {
    fn get(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
        Err(FetchError::Status(503))
    }
}

fn open(from: &Source) -> Engine {
    let fetch: Box<dyn Fetch> = match &from.answers {
        Some(answers) => Box::new(Canned::new(answers.clone())),
        None => Box::new(Offline),
    };
    Engine::open(Store::new(&from.store), fetch).replaying(from.replay)
}

fn with_munich(engine: &Engine) {
    engine.accept_notice();
    engine.add_place("München", "09162", 0).unwrap();
    engine.refresh(T0);
}

fn shipped_set(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/replay")
        .join(name)
}

fn put(set: &Path, name: &str) {
    for (file, body) in files(&shipped_set(name)) {
        std::fs::write(set.join(file), body).unwrap();
    }
}

fn files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_file())
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                std::fs::read(entry.path()).unwrap(),
            )
        })
        .collect();
    found.sort();
    found
}

#[test]
fn a_replay_start_shows_the_set_and_leaves_the_real_data_as_it_was() {
    let home = tempfile::tempdir().unwrap();
    let data = home.path();
    let live = open(&source(data, None, None));
    with_munich(&live);
    assert!(live.view(T0).places[0].warnings.is_empty());
    let before = files(data);
    assert!(before.iter().any(|(name, _)| name == "state.json"));

    let set = data.join("replay/demo");
    std::fs::create_dir_all(&set).unwrap();
    std::fs::write(set.join("091620000000.json"), DASHBOARD).unwrap();
    let from = source(data, None, Some(OsStr::new("demo")));
    let replay = open(&from);
    let fresh = replay.view(T0);
    assert!(fresh.replay);
    assert!(!fresh.notice_accepted);
    assert!(fresh.places.is_empty());
    with_munich(&replay);
    let shown = replay.view(T0);
    let headlines: Vec<&str> = shown.places[0]
        .warnings
        .iter()
        .map(|w| w.headline.as_str())
        .collect();
    assert!(
        headlines.contains(&"Amtliche WARNUNG vor SCHNEEFALL"),
        "{headlines:?}"
    );
    assert_eq!(shown.places[0].failed_since_ms, 0);

    assert_eq!(files(data), before);
    assert!(set.join("state/state.json").is_file());
    let again = open(&source(data, None, None)).view(T0);
    assert!(!again.replay);
    assert!(again.places[0].warnings.is_empty());
}

#[test]
fn a_replay_start_without_its_set_shows_a_failed_check_and_never_no_warnings() {
    let home = tempfile::tempdir().unwrap();
    let from = source(home.path(), None, Some(OsStr::new("not/a/set")));
    let replay = open(&from);
    with_munich(&replay);
    let shown = replay.view(T0);
    assert!(shown.replay);
    assert!(shown.failing);
    assert_eq!(shown.places[0].failed_since_ms, T0);
    assert!(shown.places[0].stale);
}

#[test]
fn the_weather_set_shows_every_kind_of_weather_with_its_icon() {
    let home = tempfile::tempdir().unwrap();
    let from = source(home.path(), None, Some(OsStr::new("weather")));
    std::fs::create_dir_all(from.answers.as_ref().unwrap()).unwrap();
    put(from.answers.as_ref().unwrap(), "weather");
    let replay = open(&from);
    with_munich(&replay);
    let shown = replay.view(T0);
    let mut icons: Vec<(&str, &str)> = shown.places[0]
        .warnings
        .iter()
        .map(|w| (w.id.as_str(), w.icon))
        .collect();
    icons.sort();
    assert_eq!(
        icons,
        vec![
            ("dwd.replay.dauerregen", "rain"),
            ("dwd.replay.frost", "ice"),
            ("dwd.replay.gewitter", "storm"),
            ("dwd.replay.glaette", "ice"),
            ("dwd.replay.hitze", "heat"),
            ("dwd.replay.nebel", "fog"),
            ("dwd.replay.schneefall", "snow"),
            ("dwd.replay.sturmboeen", "wind"),
            ("dwd.replay.tauwetter", "weather"),
        ]
    );
    assert!(shown.places[0].warnings.iter().all(|w| w.category == "weather"));
}

#[test]
fn the_alert_sets_in_turn_give_no_notice_then_a_new_warning_then_its_end() {
    let home = tempfile::tempdir().unwrap();
    let from = source(home.path(), None, Some(OsStr::new("alert")));
    let set = from.answers.clone().unwrap();
    std::fs::create_dir_all(&set).unwrap();
    let replay = open(&from);
    put(&set, "alert-1");
    replay.accept_notice();
    replay.add_place("München", "09162", 0).unwrap();
    assert_eq!(replay.refresh(T0), vec![]);
    assert!(replay.view(T0).places[0].warnings.is_empty());

    put(&set, "alert-2");
    let new = replay.refresh(T0 + 1);
    assert_eq!(new.len(), 1);
    assert_eq!((new[0].kind, new[0].level), (NoticeKind::New, 3));
    assert_eq!(new[0].headline, "Rauchentwicklung durch Großbrand");
    let fire = replay.warning(&new[0].id).unwrap();
    assert!(fire.detailed);
    assert!(fire.description.contains("Lagerhalle"));

    put(&set, "alert-3");
    let ended = replay.refresh(T0 + 2);
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0].kind, NoticeKind::Ended);
    let after = replay.view(T0 + 2);
    assert!(after.places[0].warnings.is_empty());
    assert_eq!(after.places[0].ended.len(), 1);
}
