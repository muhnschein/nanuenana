// SPDX-License-Identifier: GPL-3.0-or-later

use crate::geo::{distance_km, Point};
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

static PLACES: &str = include_str!("../data/de-places.tsv");
static DISTRICTS: &str = include_str!("../data/de-districts.tsv");

const STATES: &[(&str, &str)] = &[
    ("01", "Schleswig-Holstein"),
    ("02", "Hamburg"),
    ("03", "Niedersachsen"),
    ("04", "Bremen"),
    ("05", "Nordrhein-Westfalen"),
    ("06", "Hessen"),
    ("07", "Rheinland-Pfalz"),
    ("08", "Baden-Württemberg"),
    ("09", "Bayern"),
    ("10", "Saarland"),
    ("11", "Berlin"),
    ("12", "Brandenburg"),
    ("13", "Mecklenburg-Vorpommern"),
    ("14", "Sachsen"),
    ("15", "Sachsen-Anhalt"),
    ("16", "Thüringen"),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    pub name: &'static str,
    pub district: &'static str,
    pub point: Point,
    pub postcodes: &'static str,
}

pub struct Gazetteer {
    entries: Vec<Entry>,
    districts: HashMap<&'static str, &'static str>,
}

pub fn fold(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' => out.push('a'),
            'ö' => out.push('o'),
            'ü' => out.push('u'),
            'ß' => out.push_str("ss"),
            'é' | 'è' => out.push('e'),
            _ => out.push(c),
        }
    }
    out
}

pub fn state_name(district: &str) -> &'static str {
    STATES
        .iter()
        .find(|(prefix, _)| district.starts_with(prefix))
        .map_or("", |(_, name)| name)
}

fn parse_entry(line: &'static str) -> Option<Entry> {
    let mut fields = line.split('\t');
    let name = fields.next()?;
    let district = fields.next()?;
    let lat = fields.next()?.parse().ok()?;
    let lon = fields.next()?.parse().ok()?;
    let postcodes = fields.next().unwrap_or_default();
    let point = Point::new(lat, lon);
    (!name.is_empty() && district.len() == 5 && point.valid()).then_some(Entry {
        name,
        district,
        point,
        postcodes,
    })
}

fn codes(entry: &Entry) -> usize {
    entry.postcodes.split(' ').count()
}

fn rank(entry: &Entry, query: &str, digits: bool) -> Option<(u8, usize)> {
    if digits {
        let hit = entry.postcodes.split(' ').any(|code| code.starts_with(query));
        return hit.then_some((0, entry.name.len()));
    }
    let name = fold(entry.name);
    if name.starts_with(query) {
        return Some((0, name.len()));
    }
    let word_starts = name
        .split([' ', '-', '/', '(', ','])
        .skip(1)
        .any(|word| word.starts_with(query));
    word_starts.then_some((1, name.len()))
}

impl Gazetteer {
    pub fn parse(places: &'static str, districts: &'static str) -> Gazetteer {
        Gazetteer {
            entries: places.lines().filter_map(parse_entry).collect(),
            districts: districts
                .lines()
                .filter_map(|line| line.split_once('\t'))
                .collect(),
        }
    }

    pub fn germany() -> &'static Gazetteer {
        static GERMANY: OnceLock<Gazetteer> = OnceLock::new();
        GERMANY.get_or_init(|| Gazetteer::parse(PLACES, DISTRICTS))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn district_count(&self) -> usize {
        self.districts.len()
    }

    pub fn district_name(&self, district: &str) -> &'static str {
        self.districts.get(district).copied().unwrap_or_default()
    }

    pub fn find(&self, name: &str, district: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|e| e.name == name && e.district == district)
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<&Entry> {
        let query = fold(query.trim());
        if query.chars().count() < 2 {
            return Vec::new();
        }
        let digits = query.chars().all(|c| c.is_ascii_digit());
        let mut hits: Vec<((u8, usize), &Entry)> = self
            .entries
            .iter()
            .filter_map(|entry| rank(entry, &query, digits).map(|r| (r, entry)))
            .collect();
        hits.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.name.cmp(b.1.name))
                .then_with(|| codes(b.1).cmp(&codes(a.1)))
        });
        hits.into_iter().take(limit).map(|(_, entry)| entry).collect()
    }

    pub fn nearest(&self, p: Point) -> Option<(&Entry, f64)> {
        self.entries
            .iter()
            .map(|entry| (entry, distance_km(entry.point, p)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    pub fn districts_within(&self, p: Point, km: f64) -> BTreeSet<&'static str> {
        self.entries
            .iter()
            .filter(|entry| distance_km(entry.point, p) <= km)
            .map(|entry| entry.district)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_PLACES: &str = "Aach\t08335\t47.8423\t8.8573\t78267\n\
        Bad Aachen\t05334\t50.7766\t6.0834\t52062 52064\n\
        Köln\t05315\t50.9404\t6.9608\t50667 50668\n\
        Neu-Köln\t05315\t50.9000\t6.9000\t50999\n\
        broken line\n\
        Nowhere\t123\t1.0\t1.0\t00000\n\
        Offgrid\t05315\t91.0\t6.9\t50000\n";
    const SMALL_DISTRICTS: &str =
        "05315\tKreisfreie Stadt Köln\n05334\tStädteregion Aachen\n08335\tLandkreis Konstanz\n";

    fn small() -> Gazetteer {
        Gazetteer::parse(SMALL_PLACES, SMALL_DISTRICTS)
    }

    fn names(hits: Vec<&Entry>) -> Vec<&'static str> {
        hits.into_iter().map(|e| e.name).collect()
    }

    #[test]
    fn broken_lines_are_dropped() {
        let g = small();
        assert_eq!(g.len(), 4);
        assert!(!g.is_empty());
        assert_eq!(g.district_count(), 3);
        assert!(Gazetteer::parse("", "").is_empty());
    }

    #[test]
    fn folding_ignores_case_and_umlauts() {
        assert_eq!(fold("KÖLN"), "koln");
        assert_eq!(fold("Gießen"), "giessen");
        assert_eq!(fold("Müßig-Ärger"), "mussig-arger");
        assert_eq!(fold("Café"), "cafe");
    }

    #[test]
    fn search_by_name_prefix_ranks_start_first() {
        let g = small();
        assert_eq!(names(g.search("aa", 10)), vec!["Aach", "Bad Aachen"]);
        assert_eq!(names(g.search("koln", 10)), vec!["Köln", "Neu-Köln"]);
        assert_eq!(names(g.search("KÖ", 10)), vec!["Köln", "Neu-Köln"]);
        assert_eq!(names(g.search("neu", 10)), vec!["Neu-Köln"]);
        assert_eq!(names(g.search("oln", 10)), Vec::<&str>::new());
    }

    #[test]
    fn of_two_places_with_one_name_the_one_with_more_postcodes_comes_first() {
        let g = Gazetteer::parse(
            "Hamburg\t01062\t53.5312\t10.1207\t22113\n\
             Hamburg\t02000\t53.5721\t10.0138\t20095 20097 20099\n\
             Hamburg\t03999\t53.0\t10.0\t29999 29998\n",
            "",
        );
        let districts: Vec<&str> = g.search("hamb", 10).into_iter().map(|e| e.district).collect();
        assert_eq!(districts, vec!["02000", "03999", "01062"]);
    }

    #[test]
    fn search_needs_two_characters_and_honours_limit() {
        let g = small();
        assert!(g.search("a", 10).is_empty());
        assert!(g.search(" k ", 10).is_empty());
        assert_eq!(names(g.search("aa", 1)), vec!["Aach"]);
        assert_eq!(g.search("kö", 2).len(), 2);
    }

    #[test]
    fn search_by_postcode_prefix() {
        let g = small();
        assert_eq!(names(g.search("5066", 10)), vec!["Köln"]);
        assert_eq!(names(g.search("52064", 10)), vec!["Bad Aachen"]);
        assert_eq!(names(g.search("50", 10)), vec!["Köln", "Neu-Köln"]);
        assert!(g.search("0667", 10).is_empty());
    }

    #[test]
    fn find_and_district_names() {
        let g = small();
        assert_eq!(g.find("Köln", "05315").unwrap().postcodes, "50667 50668");
        assert!(g.find("Köln", "05334").is_none());
        assert!(g.find("Bonn", "05315").is_none());
        assert_eq!(g.district_name("05334"), "Städteregion Aachen");
        assert_eq!(g.district_name("99999"), "");
    }

    #[test]
    fn nearest_and_districts_within() {
        let g = small();
        let near_cologne = Point::new(50.93, 6.95);
        let (entry, km) = g.nearest(near_cologne).unwrap();
        assert_eq!(entry.name, "Köln");
        assert!(km < 2.0, "{km}");
        let around: Vec<&str> = g.districts_within(near_cologne, 10.0).into_iter().collect();
        assert_eq!(around, vec!["05315"]);
        let wide: Vec<&str> = g.districts_within(near_cologne, 70.0).into_iter().collect();
        assert_eq!(wide, vec!["05315", "05334"]);
        assert!(g.districts_within(near_cologne, 0.5).is_empty());
        assert!(Gazetteer::parse("", "").nearest(near_cologne).is_none());
    }

    #[test]
    fn states_by_district_prefix() {
        assert_eq!(state_name("09162"), "Bayern");
        assert_eq!(state_name("10044"), "Saarland");
        assert_eq!(state_name("02000"), "Hamburg");
        assert_eq!(state_name("16077"), "Thüringen");
        assert_eq!(state_name("99"), "");
    }

    #[test]
    fn bundled_germany_is_complete() {
        let g = Gazetteer::germany();
        assert!(g.len() > 14_000, "{}", g.len());
        assert_eq!(g.district_count(), 400);
        let munich = g.find("München", "09162").unwrap();
        assert!(distance_km(munich.point, Point::new(48.137, 11.575)) < 3.0);
        assert_eq!(g.district_name("09162"), "Kreisfreie Stadt München");
        assert_eq!(g.district_name("10044"), "Landkreis Saarlouis");
        assert_eq!(names(g.search("Rehlingen-S", 5)), vec!["Rehlingen-Siersburg"]);
        assert_eq!(names(g.search("66780", 5)), vec!["Rehlingen-Siersburg"]);
        assert_eq!(
            g.nearest(Point::new(48.3681, 10.8999)).map(|e| e.0.name),
            Some("Augsburg")
        );
        assert_eq!(g.district_name("09564"), "Kreisfreie Stadt Nürnberg");
        for word in ["GmbH", "Finanzamt", "Sparkasse", "Versicherung"] {
            assert!(g.search(word, 5).is_empty(), "{word}");
        }
        for entry in g.search("mün", 500) {
            assert!(!g.district_name(entry.district).is_empty(), "{}", entry.name);
            assert!(!state_name(entry.district).is_empty(), "{}", entry.name);
        }
    }
}
