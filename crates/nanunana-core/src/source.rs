// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub const REPLAY_VARIABLE: &str = "NANUNANA_REPLAY";
pub const REPLAY_FOLDER: &str = "replay";
pub const NO_SET: &str = ".none";
pub const REPLAY_STATE: &str = "state";
const LONGEST_SET_NAME: usize = 40;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub store: PathBuf,
    pub answers: Option<PathBuf>,
    pub replay: bool,
}

pub fn set_name(value: &str) -> &str {
    let plain = !value.is_empty()
        && value.len() <= LONGEST_SET_NAME
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'));
    if plain {
        value
    } else {
        NO_SET
    }
}

pub fn replay_dir(data: &Path, variable: &OsStr) -> PathBuf {
    data.join(REPLAY_FOLDER)
        .join(set_name(&variable.to_string_lossy()))
}

pub fn source(data: &Path, check_answers: Option<&Path>, variable: Option<&OsStr>) -> Source {
    match (check_answers, variable) {
        (Some(answers), variable) => Source {
            store: data.to_path_buf(),
            answers: Some(answers.to_path_buf()),
            replay: variable.is_some(),
        },
        (None, Some(variable)) => {
            let set = replay_dir(data, variable);
            Source {
                store: set.join(REPLAY_STATE),
                answers: Some(set),
                replay: true,
            }
        }
        (None, None) => Source {
            store: data.to_path_buf(),
            answers: None,
            replay: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn data() -> PathBuf {
        PathBuf::from("/home/u/.local/share/nanunana/nanunana")
    }

    #[test]
    fn without_the_variable_the_app_is_live_on_its_own_data() {
        assert_eq!(
            source(&data(), None, None),
            Source {
                store: data(),
                answers: None,
                replay: false
            }
        );
    }

    #[test]
    fn with_the_variable_answers_and_storage_come_from_the_named_set_only() {
        let variable = OsString::from("weather-kinds_1");
        let set = data().join("replay/weather-kinds_1");
        assert_eq!(
            source(&data(), None, Some(&variable)),
            Source {
                store: set.join("state"),
                answers: Some(set),
                replay: true
            }
        );
    }

    #[test]
    fn a_set_variable_never_leaves_the_app_live_whatever_it_holds() {
        for value in ["", " ", "..", "../x", "a/b", "a b", "ä", ".none", "0", "false"] {
            let variable = OsString::from(value);
            let found = source(&data(), None, Some(&variable));
            assert!(found.replay, "{value:?}");
            let answers = found.answers.expect(value);
            assert!(answers.starts_with(data().join("replay")), "{value:?}");
            assert_eq!(answers.components().count(), data().components().count() + 2);
            assert_ne!(found.store, data(), "{value:?}");
        }
    }

    #[test]
    fn only_plain_short_names_name_a_set() {
        assert_eq!(set_name("levels"), "levels");
        assert_eq!(set_name("A-1_b"), "A-1_b");
        assert_eq!(set_name(&"a".repeat(40)), "a".repeat(40));
        assert_eq!(set_name(&"a".repeat(41)), NO_SET);
        assert_eq!(set_name(""), NO_SET);
        assert_eq!(set_name("."), NO_SET);
        assert_eq!(set_name("a.b"), NO_SET);
        assert_eq!(set_name("a/b"), NO_SET);
        assert_eq!(set_name("a\\b"), NO_SET);
        assert_eq!(set_name("ö"), NO_SET);
        assert_eq!(set_name(NO_SET), NO_SET);
        assert_eq!(set_name("0"), "0");
        assert_eq!(set_name("false"), "false");
    }

    #[cfg(unix)]
    #[test]
    fn a_variable_that_is_not_text_is_no_set() {
        use std::os::unix::ffi::OsStringExt;
        let variable = OsString::from_vec(vec![b'a', 0xff, b'b']);
        assert_eq!(replay_dir(&data(), &variable), data().join("replay").join(NO_SET));
    }

    #[test]
    fn check_mode_reads_its_own_answers_and_keeps_its_storage() {
        let answers = PathBuf::from("/src/tests/qml/data");
        assert_eq!(
            source(&data(), Some(&answers), None),
            Source {
                store: data(),
                answers: Some(answers.clone()),
                replay: false
            }
        );
        let variable = OsString::from("anything");
        assert_eq!(
            source(&data(), Some(&answers), Some(&variable)),
            Source {
                store: data(),
                answers: Some(answers),
                replay: true
            }
        );
    }
}
