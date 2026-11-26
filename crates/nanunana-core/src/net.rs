// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

pub const BODY_LIMIT: u64 = 16 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(20);
const AGENT: &str = concat!("harbour-nanunana/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FetchError {
    Status(u16),
    Transport(String),
    TooLarge,
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Status(code) => write!(f, "server answered {code}"),
            FetchError::Transport(reason) => write!(f, "no answer: {reason}"),
            FetchError::TooLarge => write!(f, "answer larger than {BODY_LIMIT} bytes"),
        }
    }
}

pub trait Fetch: Send + Sync {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

pub struct Http {
    agent: ureq::Agent,
}

impl Http {
    pub fn new() -> Http {
        Http {
            agent: ureq::AgentBuilder::new()
                .timeout(TIMEOUT)
                .user_agent(AGENT)
                .redirects(2)
                .https_only(true)
                .build(),
        }
    }
}

impl Default for Http {
    fn default() -> Http {
        Http::new()
    }
}

pub fn read_limited(reader: impl Read, limit: u64) -> Result<Vec<u8>, FetchError> {
    let mut body = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut body)
        .map_err(|e| FetchError::Transport(e.to_string()))?;
    if body.len() as u64 > limit {
        return Err(FetchError::TooLarge);
    }
    Ok(body)
}

impl Fetch for Http {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        match self.agent.get(url).call() {
            Ok(response) => read_limited(response.into_reader(), BODY_LIMIT),
            Err(ureq::Error::Status(code, _)) => Err(FetchError::Status(code)),
            Err(ureq::Error::Transport(transport)) => Err(FetchError::Transport(transport.to_string())),
        }
    }
}

pub struct Canned {
    dir: PathBuf,
}

impl Canned {
    pub fn new(dir: impl Into<PathBuf>) -> Canned {
        Canned { dir: dir.into() }
    }

    pub fn file_name(url: &str) -> Option<String> {
        let name = url.rsplit('/').next().filter(|name| !name.is_empty())?;
        name.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
            .then(|| name.to_owned())
    }
}

impl Fetch for Canned {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let name = Canned::file_name(url).ok_or(FetchError::Status(404))?;
        match std::fs::read(self.dir.join(name)) {
            Ok(body) => Ok(body),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(FetchError::Status(404)),
            Err(e) => Err(FetchError::Transport(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bodies_up_to_the_limit_pass() {
        assert_eq!(read_limited(&b"abcd"[..], 4), Ok(b"abcd".to_vec()));
        assert_eq!(read_limited(&b""[..], 0), Ok(Vec::new()));
    }

    #[test]
    fn bodies_over_the_limit_fail() {
        assert_eq!(read_limited(&b"abcde"[..], 4), Err(FetchError::TooLarge));
        assert_eq!(read_limited(&b"a"[..], 0), Err(FetchError::TooLarge));
    }

    #[test]
    fn errors_read_plainly() {
        assert_eq!(FetchError::Status(503).to_string(), "server answered 503");
        assert_eq!(
            FetchError::Transport("dns".to_owned()).to_string(),
            "no answer: dns"
        );
        assert!(FetchError::TooLarge.to_string().contains("16777216"));
    }

    #[test]
    fn plain_http_is_refused() {
        let error = Http::new().get("http://127.0.0.1:9/x").unwrap_err();
        assert!(matches!(error, FetchError::Transport(_)), "{error:?}");
    }

    #[test]
    fn canned_answers_come_from_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("091620000000.json"), b"[]").unwrap();
        let canned = Canned::new(dir.path());
        assert_eq!(
            canned.get("https://example.org/api31/dashboard/091620000000.json"),
            Ok(b"[]".to_vec())
        );
        assert_eq!(
            canned.get("https://example.org/api31/warnings/none.json"),
            Err(FetchError::Status(404))
        );
        assert_eq!(
            canned.get("https://example.org/api31/"),
            Err(FetchError::Status(404))
        );
        assert_eq!(
            canned.get("https://example.org/x/..%2Fsecret"),
            Err(FetchError::Status(404))
        );
    }

    #[test]
    fn canned_names_are_plain_file_names() {
        assert_eq!(Canned::file_name("https://h/a/b.json").as_deref(), Some("b.json"));
        assert_eq!(
            Canned::file_name("https://h/a/mow.DE-BY-1_2.geojson").as_deref(),
            Some("mow.DE-BY-1_2.geojson")
        );
        assert_eq!(Canned::file_name("https://h/a/"), None);
        assert_eq!(Canned::file_name("https://h/a/b c"), None);
        assert_eq!(Canned::file_name("https://h/a/b?c"), None);
    }
}
