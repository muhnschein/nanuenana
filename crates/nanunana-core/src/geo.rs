// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

const EARTH_RADIUS_KM: f64 = 6371.0088;
const KM_PER_DEGREE: f64 = EARTH_RADIUS_KM * std::f64::consts::PI / 180.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

impl Point {
    pub fn new(lat: f64, lon: f64) -> Point {
        Point { lat, lon }
    }

    pub fn valid(self) -> bool {
        self.lat.is_finite() && self.lon.is_finite() && self.lat.abs() <= 90.0 && self.lon.abs() <= 180.0
    }
}

pub type Ring = Vec<Point>;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub outer: Ring,
    pub holes: Vec<Ring>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub polygons: Vec<Polygon>,
}

pub fn distance_km(a: Point, b: Point) -> f64 {
    let (lat_a, lat_b) = (a.lat.to_radians(), b.lat.to_radians());
    let half_lat = (lat_b - lat_a) / 2.0;
    let half_lon = (b.lon - a.lon).to_radians() / 2.0;
    let h = half_lat.sin().powi(2) + lat_a.cos() * lat_b.cos() * half_lon.sin().powi(2);
    2.0 * EARTH_RADIUS_KM * h.sqrt().min(1.0).asin()
}

fn local(origin: Point, p: Point) -> (f64, f64) {
    let x = (p.lon - origin.lon) * origin.lat.to_radians().cos() * KM_PER_DEGREE;
    let y = (p.lat - origin.lat) * KM_PER_DEGREE;
    (x, y)
}

fn segment_distance_km(origin: Point, a: Point, b: Point) -> f64 {
    let (ax, ay) = local(origin, a);
    let (bx, by) = local(origin, b);
    let (dx, dy) = (bx - ax, by - ay);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (-(ax * dx + ay * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (ax + t * dx).hypot(ay + t * dy)
}

fn ring_contains(ring: &[Point], p: Point) -> bool {
    let mut inside = false;
    let mut previous = match ring.last() {
        Some(last) => *last,
        None => return false,
    };
    for &current in ring {
        let crosses = (current.lat > p.lat) != (previous.lat > p.lat);
        if crosses {
            let at = (previous.lon - current.lon) * (p.lat - current.lat) / (previous.lat - current.lat)
                + current.lon;
            if p.lon < at {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

fn ring_distance_km(ring: &[Point], p: Point) -> Option<f64> {
    let mut best: Option<f64> = None;
    let mut previous = *ring.last()?;
    for &current in ring {
        let d = segment_distance_km(p, previous, current);
        best = Some(best.map_or(d, |b| b.min(d)));
        previous = current;
    }
    best
}

impl Polygon {
    pub fn contains(&self, p: Point) -> bool {
        ring_contains(&self.outer, p) && !self.holes.iter().any(|hole| ring_contains(hole, p))
    }

    pub fn distance_km(&self, p: Point) -> Option<f64> {
        if self.contains(p) {
            return Some(0.0);
        }
        std::iter::once(&self.outer)
            .chain(self.holes.iter())
            .filter_map(|ring| ring_distance_km(ring, p))
            .reduce(f64::min)
    }
}

impl Shape {
    pub fn is_empty(&self) -> bool {
        self.polygons.iter().all(|polygon| polygon.outer.is_empty())
    }

    pub fn contains(&self, p: Point) -> bool {
        self.polygons.iter().any(|polygon| polygon.contains(p))
    }

    pub fn distance_km(&self, p: Point) -> Option<f64> {
        self.polygons
            .iter()
            .filter_map(|polygon| polygon.distance_km(p))
            .reduce(f64::min)
    }

    pub fn within_km(&self, p: Point, km: f64) -> bool {
        self.distance_km(p).is_some_and(|d| d <= km)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(lat: f64, lon: f64, half: f64) -> Ring {
        vec![
            Point::new(lat - half, lon - half),
            Point::new(lat - half, lon + half),
            Point::new(lat + half, lon + half),
            Point::new(lat + half, lon - half),
        ]
    }

    fn close(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() <= tolerance
    }

    #[test]
    fn haversine_matches_known_distances() {
        let munich = Point::new(48.1372, 11.5756);
        let berlin = Point::new(52.5200, 13.4050);
        assert!(close(distance_km(munich, berlin), 504.4, 1.0));
        assert_eq!(distance_km(munich, munich), 0.0);
        let north = Point::new(49.1372, 11.5756);
        assert!(close(distance_km(munich, north), 111.19, 0.05));
        let east = Point::new(48.1372, 12.5756);
        assert!(close(distance_km(munich, east), 74.2, 0.1));
    }

    #[test]
    fn point_validity() {
        assert!(Point::new(48.0, 11.0).valid());
        assert!(Point::new(-90.0, 180.0).valid());
        assert!(!Point::new(90.1, 0.0).valid());
        assert!(!Point::new(0.0, -180.1).valid());
        assert!(!Point::new(f64::NAN, 0.0).valid());
        assert!(!Point::new(0.0, f64::INFINITY).valid());
    }

    #[test]
    fn ring_containment() {
        let polygon = Polygon {
            outer: square(48.0, 11.0, 0.1),
            holes: vec![],
        };
        assert!(polygon.contains(Point::new(48.0, 11.0)));
        assert!(polygon.contains(Point::new(48.09, 10.91)));
        assert!(!polygon.contains(Point::new(48.11, 11.0)));
        assert!(!polygon.contains(Point::new(48.0, 11.11)));
        assert!(!polygon.contains(Point::new(47.89, 11.0)));
        assert!(!polygon.contains(Point::new(48.0, 10.89)));
    }

    #[test]
    fn slanted_edges_are_followed() {
        let polygon = Polygon {
            outer: vec![
                Point::new(48.0, 11.0),
                Point::new(48.4, 11.2),
                Point::new(48.0, 11.8),
            ],
            holes: vec![],
        };
        assert!(polygon.contains(Point::new(48.2, 11.12)));
        assert!(!polygon.contains(Point::new(48.2, 11.08)));
        assert!(polygon.contains(Point::new(48.2, 11.48)));
        assert!(!polygon.contains(Point::new(48.2, 11.52)));
        assert!(polygon.contains(Point::new(48.3, 11.17)));
        assert!(!polygon.contains(Point::new(48.3, 11.13)));
        assert!(polygon.contains(Point::new(48.3, 11.33)));
        assert!(!polygon.contains(Point::new(48.3, 11.37)));
    }

    #[test]
    fn western_edge_counts_as_inside_and_eastern_edge_as_outside() {
        let polygon = Polygon {
            outer: vec![
                Point::new(48.0, 11.0),
                Point::new(49.0, 11.0),
                Point::new(49.0, 12.0),
                Point::new(48.0, 12.0),
            ],
            holes: vec![],
        };
        assert!(polygon.contains(Point::new(48.5, 11.0)));
        assert!(!polygon.contains(Point::new(48.5, 12.0)));
    }

    #[test]
    fn concave_ring_containment() {
        let outer = vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 3.0),
            Point::new(3.0, 3.0),
            Point::new(3.0, 2.0),
            Point::new(1.0, 2.0),
            Point::new(1.0, 1.0),
            Point::new(3.0, 1.0),
            Point::new(3.0, 0.0),
        ];
        let polygon = Polygon { outer, holes: vec![] };
        assert!(polygon.contains(Point::new(0.5, 1.5)));
        assert!(polygon.contains(Point::new(2.0, 0.5)));
        assert!(polygon.contains(Point::new(2.0, 2.5)));
        assert!(!polygon.contains(Point::new(2.0, 1.5)));
    }

    #[test]
    fn hole_is_outside() {
        let polygon = Polygon {
            outer: square(48.0, 11.0, 0.2),
            holes: vec![square(48.0, 11.0, 0.1)],
        };
        assert!(!polygon.contains(Point::new(48.0, 11.0)));
        assert!(polygon.contains(Point::new(48.15, 11.0)));
        let d = polygon.distance_km(Point::new(48.0, 11.0)).unwrap();
        assert!(close(d, 0.1 * 74.4, 0.2), "{d}");
    }

    #[test]
    fn distance_to_polygon() {
        let polygon = Polygon {
            outer: square(48.0, 11.0, 0.1),
            holes: vec![],
        };
        assert_eq!(polygon.distance_km(Point::new(48.0, 11.0)), Some(0.0));
        let north = polygon.distance_km(Point::new(48.2, 11.0)).unwrap();
        assert!(close(north, 11.12, 0.05), "{north}");
        let east = polygon.distance_km(Point::new(48.0, 11.2)).unwrap();
        assert!(close(east, 7.44, 0.05), "{east}");
        let corner = polygon.distance_km(Point::new(48.2, 11.2)).unwrap();
        assert!(close(corner, 11.12f64.hypot(7.42), 0.1), "{corner}");
    }

    #[test]
    fn distance_to_segment_end_and_middle() {
        let a = Point::new(48.0, 11.0);
        let b = Point::new(48.0, 11.2);
        let above_middle = segment_distance_km(Point::new(48.1, 11.1), a, b);
        assert!(close(above_middle, 11.12, 0.05), "{above_middle}");
        let beyond_end = segment_distance_km(Point::new(48.0, 11.3), a, b);
        assert!(close(beyond_end, 7.44, 0.05), "{beyond_end}");
        let before_start = segment_distance_km(Point::new(48.0, 10.9), a, b);
        assert!(close(before_start, 7.44, 0.05), "{before_start}");
        let on_point = segment_distance_km(Point::new(48.1, 11.0), a, a);
        assert!(close(on_point, 11.12, 0.05), "{on_point}");
    }

    #[test]
    fn empty_shapes_have_no_distance() {
        let shape = Shape::default();
        assert!(shape.is_empty());
        assert_eq!(shape.distance_km(Point::new(0.0, 0.0)), None);
        assert!(!shape.within_km(Point::new(0.0, 0.0), 1000.0));
        assert!(!shape.contains(Point::new(0.0, 0.0)));
        let hollow = Shape {
            polygons: vec![Polygon::default()],
        };
        assert!(hollow.is_empty());
        assert_eq!(hollow.distance_km(Point::new(0.0, 0.0)), None);
    }

    #[test]
    fn shape_uses_nearest_polygon() {
        let shape = Shape {
            polygons: vec![
                Polygon {
                    outer: square(50.0, 11.0, 0.1),
                    holes: vec![],
                },
                Polygon {
                    outer: square(48.0, 11.0, 0.1),
                    holes: vec![],
                },
            ],
        };
        assert!(!shape.is_empty());
        let p = Point::new(48.2, 11.0);
        let d = shape.distance_km(p).unwrap();
        assert!(close(d, 11.12, 0.05), "{d}");
        assert!(shape.within_km(p, 11.2));
        assert!(!shape.within_km(p, 11.0));
        assert!(shape.contains(Point::new(50.0, 11.0)));
        assert!(shape.within_km(Point::new(50.0, 11.0), 0.0));
    }
}
