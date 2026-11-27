// SPDX-License-Identifier: GPL-3.0-or-later

use nanunana_core::engine::Engine;
use nanunana_core::net::{Canned, Fetch, FetchError};
use nanunana_core::source::{source, Source};
use nanunana_core::store::Store;
use std::ffi::OsStr;
use std::path::Path;

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
