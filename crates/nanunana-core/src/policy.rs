// SPDX-License-Identifier: GPL-3.0-or-later

pub const SUPPORTED: &[&str] = &["DE"];
pub const BLOCKED: &[&str] = &["RU", "BY", "IR", "KP", "SY"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offer {
    Supported,
    NotYet,
    Never,
}

pub fn offer_from(country: &str, supported: &[&str], blocked: &[&str]) -> Offer {
    let code = country.trim().to_ascii_uppercase();
    if blocked.contains(&code.as_str()) {
        Offer::Never
    } else if supported.contains(&code.as_str()) {
        Offer::Supported
    } else {
        Offer::NotYet
    }
}

pub fn offer(country: &str) -> Offer {
    offer_from(country, SUPPORTED, BLOCKED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn germany_is_supported() {
        assert_eq!(offer("DE"), Offer::Supported);
        assert_eq!(offer(" de "), Offer::Supported);
    }

    #[test]
    fn blocked_countries_are_never_offered() {
        for code in ["RU", "BY", "IR", "KP", "SY", "ru", " by "] {
            assert_eq!(offer(code), Offer::Never, "{code}");
        }
        assert_eq!(BLOCKED.len(), 5);
    }

    #[test]
    fn other_countries_are_not_offered_yet() {
        for code in ["FR", "UA", "IL", "US", "", "??"] {
            assert_eq!(offer(code), Offer::NotYet, "{code}");
        }
    }

    #[test]
    fn no_country_is_both_supported_and_blocked() {
        for code in SUPPORTED {
            assert!(!BLOCKED.contains(code), "{code}");
        }
    }

    #[test]
    fn blocked_wins_over_supported() {
        assert_eq!(offer_from("RU", &["RU", "DE"], &["RU"]), Offer::Never);
        assert_eq!(offer_from("DE", &["RU", "DE"], &["RU"]), Offer::Supported);
        assert_eq!(offer_from("FR", &["RU", "DE"], &["RU"]), Offer::NotYet);
    }
}
