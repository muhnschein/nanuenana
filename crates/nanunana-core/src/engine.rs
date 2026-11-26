// SPDX-License-Identifier: GPL-3.0-or-later

use crate::alerts::{decide, failure_due, Notice, SeenMap, Sighting};
use crate::bbk;
use crate::gazetteer::{state_name, Gazetteer};
use crate::geo::{distance_km, Point};
use crate::model::{Cached, Kind, Summary};
use crate::net::Fetch;
use crate::policy::{offer, Offer};
use crate::settings::Settings;
use crate::store::Store;
use crate::view::{self, PlaceView, SearchHit, View, WarningView};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard, PoisonError};

pub const COUNTRY: &str = "DE";
pub const LOCATION_ID: u32 = 0;
pub const LOCATION_KM: u8 = 2;
pub const MARGIN_KM: f64 = 10.0;
pub const OUTSIDE_KM: f64 = 25.0;
pub const MAX_KM: u8 = 25;
pub const SEARCH_LIMIT: usize = 30;
const STATE_FILE: &str = "state.json";
const CACHE_FILE: &str = "cache.json";
const SLACK_MS: i64 = 60_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Place {
    pub id: u32,
    pub name: String,
    pub district: String,
    pub lat: f64,
    pub lon: f64,
    pub km: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct District {
    fetched_ms: i64,
    failed_ms: i64,
    warnings: Vec<Summary>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fix {
    point: Point,
    at_ms: i64,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct State {
    notice_accepted: bool,
    next_id: u32,
    places: Vec<Place>,
    settings: Settings,
    seen: SeenMap,
    primed: BTreeSet<u32>,
    districts: BTreeMap<String, District>,
    failing_since_ms: i64,
    failure_notified: bool,
    checked_ms: i64,
    #[serde(skip)]
    fix: Option<Fix>,
    #[serde(skip)]
    locale: String,
}

type Cache = BTreeMap<String, Cached>;

#[derive(Clone, Debug, PartialEq)]
struct Target {
    id: u32,
    name: String,
    district: String,
    centre: Point,
    km: u8,
    districts: BTreeSet<String>,
    fix_ms: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddError {
    Unknown,
    Duplicate,
    NotOffered,
}

pub struct Engine {
    store: Store,
    fetch: Box<dyn Fetch>,
    gazetteer: &'static Gazetteer,
    state: Mutex<State>,
    cache: Mutex<Cache>,
    refreshing: Mutex<()>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn around(gazetteer: &Gazetteer, centre: Point, km: u8, own: &str) -> BTreeSet<String> {
    let mut districts: BTreeSet<String> = if km == 0 {
        BTreeSet::new()
    } else {
        gazetteer
            .districts_within(centre, f64::from(km) + MARGIN_KM)
            .into_iter()
            .map(str::to_owned)
            .collect()
    };
    districts.insert(own.to_owned());
    districts
}

fn matches(target: &Target, summary: &Summary, cache: &Cache) -> bool {
    if target.km == 0 {
        return true;
    }
    let area = cache
        .get(&summary.id)
        .filter(|cached| cached.version == summary.version)
        .and_then(|cached| cached.area.as_ref())
        .filter(|area| !area.is_empty());
    area.is_none_or(|area| area.within_km(target.centre, f64::from(target.km)))
}

fn sighted<'a>(
    target: &Target,
    districts: &'a BTreeMap<String, District>,
    cache: &Cache,
) -> Vec<&'a Summary> {
    let mut found: Vec<&Summary> = Vec::new();
    for summary in target
        .districts
        .iter()
        .filter_map(|d| districts.get(d))
        .flat_map(|d| d.warnings.iter())
    {
        if !found.iter().any(|known| known.id == summary.id) && matches(target, summary, cache) {
            found.push(summary);
        }
    }
    found
}

impl State {
    fn language(&self) -> &str {
        if self.settings.language.is_empty() {
            &self.locale
        } else {
            &self.settings.language
        }
    }

    fn location_target(&self, gazetteer: &Gazetteer) -> Option<Target> {
        let fix = self.fix.filter(|_| self.settings.location)?;
        let (entry, km) = gazetteer.nearest(fix.point)?;
        (km <= OUTSIDE_KM).then(|| Target {
            id: LOCATION_ID,
            name: entry.name.to_owned(),
            district: entry.district.to_owned(),
            centre: fix.point,
            km: LOCATION_KM,
            districts: around(gazetteer, fix.point, LOCATION_KM, entry.district),
            fix_ms: fix.at_ms,
        })
    }

    fn targets(&self, gazetteer: &Gazetteer) -> Vec<Target> {
        let saved = self.places.iter().map(|place| {
            let centre = Point::new(place.lat, place.lon);
            Target {
                id: place.id,
                name: place.name.clone(),
                district: place.district.clone(),
                centre,
                km: place.km,
                districts: around(gazetteer, centre, place.km, &place.district),
                fix_ms: 0,
            }
        });
        self.location_target(gazetteer).into_iter().chain(saved).collect()
    }

    fn location_state(&self, gazetteer: &Gazetteer) -> &'static str {
        if !self.settings.location {
            return "off";
        }
        match self.fix {
            None => "waiting",
            Some(_) if self.location_target(gazetteer).is_none() => "outside",
            Some(_) => "on",
        }
    }

    fn stale_after_ms(&self) -> i64 {
        2 * self.settings.interval_ms() + SLACK_MS
    }
}

impl Engine {
    pub fn open(store: Store, fetch: Box<dyn Fetch>) -> Engine {
        let state: State = store.load(STATE_FILE);
        let cache: Cache = store.load(CACHE_FILE);
        Engine {
            store,
            fetch,
            gazetteer: Gazetteer::germany(),
            state: Mutex::new(state),
            cache: Mutex::new(cache),
            refreshing: Mutex::new(()),
        }
    }

    fn save_state(&self, state: &State) {
        if let Err(e) = self.store.save(STATE_FILE, state) {
            log::error!("cannot save {STATE_FILE}: {e}");
        }
    }

    fn save_cache(&self, cache: &Cache) {
        if let Err(e) = self.store.save(CACHE_FILE, cache) {
            log::error!("cannot save {CACHE_FILE}: {e}");
        }
    }

    pub fn notice_accepted(&self) -> bool {
        lock(&self.state).notice_accepted
    }

    pub fn accept_notice(&self) {
        let mut state = lock(&self.state);
        state.notice_accepted = true;
        self.save_state(&state);
    }

    pub fn set_locale(&self, locale: &str) {
        lock(&self.state).locale = locale.to_owned();
    }

    pub fn settings(&self) -> Settings {
        lock(&self.state).settings.clone()
    }

    pub fn set_setting(&self, key: &str, value: serde_json::Value) -> bool {
        let mut state = lock(&self.state);
        let Some(changed) = state.settings.with(key, value) else {
            return false;
        };
        if !changed.location {
            state.fix = None;
            state.primed.remove(&LOCATION_ID);
        }
        state.settings = changed;
        self.save_state(&state);
        true
    }

    pub fn search(&self, query: &str) -> Vec<SearchHit> {
        let state = lock(&self.state);
        let trimmed = query.trim();
        self.gazetteer
            .search(trimmed, SEARCH_LIMIT)
            .into_iter()
            .map(|entry| SearchHit {
                name: entry.name.to_owned(),
                district: entry.district.to_owned(),
                district_name: self.gazetteer.district_name(entry.district).to_owned(),
                state_name: state_name(entry.district).to_owned(),
                postcode: entry
                    .postcodes
                    .split(' ')
                    .find(|code| code.starts_with(trimmed))
                    .or_else(|| entry.postcodes.split(' ').next())
                    .unwrap_or_default()
                    .to_owned(),
                saved: state
                    .places
                    .iter()
                    .any(|p| p.name == entry.name && p.district == entry.district),
            })
            .collect()
    }

    pub fn add_place(&self, name: &str, district: &str, km: u8) -> Result<u32, AddError> {
        if offer(COUNTRY) != Offer::Supported {
            return Err(AddError::NotOffered);
        }
        let entry = self.gazetteer.find(name, district).ok_or(AddError::Unknown)?;
        let mut state = lock(&self.state);
        if state
            .places
            .iter()
            .any(|p| p.name == name && p.district == district)
        {
            return Err(AddError::Duplicate);
        }
        state.next_id += 1;
        let id = state.next_id;
        state.places.push(Place {
            id,
            name: entry.name.to_owned(),
            district: entry.district.to_owned(),
            lat: entry.point.lat,
            lon: entry.point.lon,
            km: km.min(MAX_KM),
        });
        self.save_state(&state);
        Ok(id)
    }

    pub fn set_area(&self, id: u32, km: u8) -> bool {
        let mut state = lock(&self.state);
        let Some(place) = state.places.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        place.km = km.min(MAX_KM);
        state.primed.remove(&id);
        self.save_state(&state);
        true
    }

    pub fn remove_place(&self, id: u32) -> bool {
        let mut state = lock(&self.state);
        let before = state.places.len();
        state.places.retain(|p| p.id != id);
        if state.places.len() == before {
            return false;
        }
        state.primed.remove(&id);
        self.save_state(&state);
        true
    }

    pub fn places(&self) -> Vec<Place> {
        lock(&self.state).places.clone()
    }

    pub fn set_position(&self, lat: f64, lon: f64, now_ms: i64) -> bool {
        let point = Point::new(lat, lon);
        let mut state = lock(&self.state);
        if !state.settings.location || !point.valid() {
            return false;
        }
        let moved = state.fix.is_none_or(|fix| distance_km(fix.point, point) > 0.05);
        state.fix = Some(Fix { point, at_ms: now_ms });
        moved
    }

    fn dashboard(&self, district: &str) -> Option<Vec<Summary>> {
        let url = bbk::dashboard_url(district)?;
        let bytes = self
            .fetch
            .get(&url)
            .map_err(|e| log::warn!("dashboard {district}: {e}"))
            .ok()?;
        bbk::parse_dashboard(&bytes)
            .map_err(|e| log::warn!("dashboard {district} unreadable: {e}"))
            .ok()
    }

    fn fill(&self, summary: &Summary, known: Option<Cached>) -> Cached {
        let mut cached = known
            .filter(|c| c.version == summary.version)
            .unwrap_or_else(|| Cached {
                version: summary.version,
                ..Cached::default()
            });
        if cached.detail.is_none() {
            cached.detail = bbk::detail_url(&summary.id)
                .and_then(|url| self.fetch.get(&url).ok())
                .and_then(|bytes| bbk::parse_detail(&bytes).ok());
        }
        if cached.area.is_none() {
            cached.area = bbk::area_url(&summary.id)
                .and_then(|url| self.fetch.get(&url).ok())
                .and_then(|bytes| bbk::parse_area(&bytes).ok());
        }
        cached
    }

    fn refresh_cache(&self, lists: &BTreeMap<String, Option<Vec<Summary>>>) {
        let known = lock(&self.cache).clone();
        let mut filled: Cache = Cache::new();
        for summary in lists.values().flatten().flatten() {
            if filled.contains_key(&summary.id) {
                continue;
            }
            let before = known.get(&summary.id).cloned();
            let complete = before
                .as_ref()
                .is_some_and(|c| c.version == summary.version && c.detail.is_some() && c.area.is_some());
            if !complete {
                filled.insert(summary.id.clone(), self.fill(summary, before));
            }
        }
        if filled.is_empty() {
            return;
        }
        let mut cache = lock(&self.cache);
        cache.extend(filled);
        self.save_cache(&cache);
    }

    fn prune_cache(&self, state: &State) {
        let mut cache = lock(&self.cache);
        let before = cache.len();
        cache.retain(|id, _| {
            state
                .districts
                .values()
                .any(|d| d.warnings.iter().any(|w| &w.id == id))
        });
        if cache.len() != before {
            self.save_cache(&cache);
        }
    }

    pub fn refresh(&self, now_ms: i64) -> Vec<Notice> {
        let _one_at_a_time = lock(&self.refreshing);
        let needed: BTreeSet<String> = {
            let state = lock(&self.state);
            if !state.notice_accepted {
                return Vec::new();
            }
            state
                .targets(self.gazetteer)
                .into_iter()
                .flat_map(|t| t.districts)
                .collect()
        };
        let lists: BTreeMap<String, Option<Vec<Summary>>> =
            needed.iter().map(|d| (d.clone(), self.dashboard(d))).collect();
        self.refresh_cache(&lists);

        let mut guard = lock(&self.state);
        let state = &mut *guard;
        for (district, list) in lists {
            let entry = state.districts.entry(district).or_default();
            match list {
                Some(warnings) => {
                    entry.fetched_ms = now_ms;
                    entry.failed_ms = 0;
                    entry.warnings = warnings;
                }
                None if entry.failed_ms == 0 => entry.failed_ms = now_ms,
                None => {}
            }
        }
        let targets = state.targets(self.gazetteer);
        let wanted: BTreeSet<&String> = targets.iter().flat_map(|t| t.districts.iter()).collect();
        state.districts.retain(|d, _| wanted.contains(d));
        let fresh = |t: &Target| {
            t.districts.iter().all(|d| {
                state
                    .districts
                    .get(d)
                    .is_some_and(|s| s.failed_ms == 0 && s.fetched_ms == now_ms)
            })
        };
        let complete = targets.iter().all(fresh);
        let checked: Vec<u32> = targets.iter().filter(|t| fresh(t)).map(|t| t.id).collect();

        let mut notices = {
            let cache = lock(&self.cache);
            let sightings: Vec<Sighting> = targets
                .iter()
                .flat_map(|target| {
                    let quiet = !state.primed.contains(&target.id);
                    sighted(target, &state.districts, &cache)
                        .into_iter()
                        .map(move |summary| Sighting {
                            summary,
                            place: &target.name,
                            quiet,
                        })
                })
                .collect();
            decide(&mut state.seen, &sightings, complete, &state.settings)
        };
        state.primed.retain(|id| targets.iter().any(|t| t.id == *id));
        state.primed.extend(checked);

        if complete {
            state.failing_since_ms = 0;
            state.failure_notified = false;
            state.checked_ms = now_ms;
        } else {
            if state.failing_since_ms == 0 {
                state.failing_since_ms = now_ms;
            }
            if failure_due(
                state.failing_since_ms,
                now_ms,
                state.failure_notified,
                &state.settings,
            ) {
                state.failure_notified = true;
                notices.push(Notice::failure());
            }
        }
        self.save_state(state);
        self.prune_cache(state);
        notices
    }

    fn place_view(&self, target: &Target, state: &State, cache: &Cache, now_ms: i64) -> PlaceView {
        let language = state.language();
        let found = sighted(target, &state.districts, cache);
        let (ended, active): (Vec<&Summary>, Vec<&Summary>) =
            found.into_iter().partition(|s| s.kind == Kind::Cancel);
        let rows = |list: Vec<&Summary>| {
            let mut rows: Vec<_> = list
                .into_iter()
                .map(|s| view::row(s, cache.get(&s.id), language))
                .collect();
            view::sort_rows(&mut rows);
            rows
        };
        let warnings = rows(active);
        let known: Vec<&District> = target
            .districts
            .iter()
            .filter_map(|d| state.districts.get(d))
            .collect();
        let all_known = known.len() == target.districts.len();
        let updated_ms = if all_known {
            known.iter().map(|d| d.fetched_ms).min().unwrap_or_default()
        } else {
            0
        };
        let failed_since_ms = known
            .iter()
            .map(|d| d.failed_ms)
            .filter(|ms| *ms > 0)
            .min()
            .unwrap_or_default();
        let lost = target.fix_ms > 0 && now_ms - target.fix_ms > state.stale_after_ms();
        PlaceView {
            id: target.id,
            name: target.name.clone(),
            district_name: self.gazetteer.district_name(&target.district).to_owned(),
            state_name: state_name(&target.district).to_owned(),
            km: target.km,
            current: target.id == LOCATION_ID,
            lost_since_ms: if lost { target.fix_ms } else { 0 },
            updated_ms,
            failed_since_ms,
            stale: updated_ms == 0 || failed_since_ms > 0 || now_ms - updated_ms > state.stale_after_ms(),
            level: warnings.first().map_or(0, |w| w.level),
            headline: warnings.first().map(|w| w.headline.clone()).unwrap_or_default(),
            warnings,
            ended: rows(ended),
        }
    }

    pub fn view(&self, now_ms: i64) -> View {
        let state = lock(&self.state);
        let cache = lock(&self.cache);
        View {
            notice_accepted: state.notice_accepted,
            checked_ms: state.checked_ms,
            failing: state.failing_since_ms > 0,
            location: state.location_state(self.gazetteer),
            places: state
                .targets(self.gazetteer)
                .iter()
                .map(|target| self.place_view(target, &state, &cache, now_ms))
                .collect(),
        }
    }

    pub fn warning(&self, id: &str) -> Option<WarningView> {
        let state = lock(&self.state);
        let cache = lock(&self.cache);
        let targets = state.targets(self.gazetteer);
        let (target, summary) = targets.iter().find_map(|target| {
            sighted(target, &state.districts, &cache)
                .into_iter()
                .find(|s| s.id == id)
                .map(|s| (target, s))
        })?;
        let mut shown = view::warning(summary, cache.get(id), state.language());
        shown.place = target.name.clone();
        shown.updated_ms = target
            .districts
            .iter()
            .filter_map(|d| state.districts.get(d))
            .map(|d| d.fetched_ms)
            .min()
            .unwrap_or_default();
        Some(shown)
    }
}
