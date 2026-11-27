// SPDX-License-Identifier: GPL-3.0-or-later

use nanunana_core::engine::Engine;
use nanunana_core::net::Fetch;
use nanunana_core::store::Store;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

static ENGINE: OnceLock<Arc<Engine>> = OnceLock::new();

pub fn open(dir: PathBuf, fetch: Box<dyn Fetch>, replay: bool) -> Arc<Engine> {
    ENGINE
        .get_or_init(|| Arc::new(Engine::open(Store::new(dir), fetch).replaying(replay)))
        .clone()
}

pub fn engine() -> Option<Arc<Engine>> {
    ENGINE.get().cloned()
}
