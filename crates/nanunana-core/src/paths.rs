// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const ORGANIZATION: &str = "nanunana";
pub const APPLICATION: &str = "nanunana";

pub fn data_dir_from(home: Option<OsString>) -> PathBuf {
    home.map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".local/share")
        .join(ORGANIZATION)
        .join(APPLICATION)
}

pub fn data_dir() -> PathBuf {
    data_dir_from(std::env::var_os("HOME"))
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_lives_in_the_sandbox_folder() {
        assert_eq!(
            data_dir_from(Some("/home/defaultuser".into())),
            PathBuf::from("/home/defaultuser/.local/share/nanunana/nanunana")
        );
        assert_eq!(
            data_dir_from(None),
            PathBuf::from("/.local/share/nanunana/nanunana")
        );
    }

    #[test]
    fn the_clock_is_past_2025() {
        assert!(now_ms() > 1_735_689_600_000);
    }
}
