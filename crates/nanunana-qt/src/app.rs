// SPDX-License-Identifier: GPL-3.0-or-later

use crate::runtime;
use nanunana_core::alerts::Notice;
use nanunana_core::engine::{AddError, Engine, MAX_KM};
use nanunana_core::paths::now_ms;
use qmetaobject::prelude::*;
use qmetaobject::{QPointer, QSingletonInit};
use serde::Serialize;
use std::sync::Arc;

#[derive(QObject, Default)]
pub struct App {
    base: qt_base_class!(trait QObject),
    version: qt_property!(QString; CONST),
    stateJson: qt_property!(QString; NOTIFY stateChanged),
    stateChanged: qt_signal!(),
    settingsJson: qt_property!(QString; NOTIFY settingsChanged),
    settingsChanged: qt_signal!(),
    checking: qt_property!(bool; NOTIFY checkingChanged),
    checkingChanged: qt_signal!(),
    checked: qt_signal!(notices: QString),
    checkRequested: qt_signal!(),

    acceptNotice: qt_method!(fn(&mut self)),
    setSetting: qt_method!(fn(&mut self, key: QString, value_json: QString) -> bool),
    search: qt_method!(fn(&self, query: QString) -> QString),
    addPlace: qt_method!(fn(&mut self, name: QString, district: QString, km: i32) -> QString),
    setArea: qt_method!(fn(&mut self, id: i32, km: i32) -> bool),
    removePlace: qt_method!(fn(&mut self, id: i32) -> bool),
    setPosition: qt_method!(fn(&mut self, lat: f64, lon: f64) -> bool),
    check: qt_method!(fn(&mut self)),
    requestCheck: qt_method!(fn(&self)),
    reload: qt_method!(fn(&mut self)),
    warningJson: qt_method!(fn(&self, id: QString) -> QString),

    engine: Option<Arc<Engine>>,
}

impl QSingletonInit for App {
    fn init(&mut self) {
        self.version = QString::from(env!("CARGO_PKG_VERSION"));
        self.engine = runtime::engine();
        if let Some(engine) = &self.engine {
            engine.set_locale(&crate::system_locale());
        }
        self.stateJson = QString::from(self.state_text().as_str());
        self.settingsJson = QString::from(self.settings_text().as_str());
    }
}

pub fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}

pub fn area_km(km: i32) -> Option<u8> {
    u8::try_from(km).ok().filter(|km| *km <= MAX_KM)
}

pub fn place_id(id: i32) -> Option<u32> {
    u32::try_from(id).ok().filter(|id| *id > 0)
}

pub fn add_error_key(error: AddError) -> &'static str {
    match error {
        AddError::Unknown => "unknown",
        AddError::Duplicate => "duplicate",
        AddError::NotOffered => "notOffered",
    }
}

impl App {
    fn state_text(&self) -> String {
        match &self.engine {
            Some(engine) => to_json(&engine.view(now_ms())),
            None => "null".to_owned(),
        }
    }

    fn settings_text(&self) -> String {
        match &self.engine {
            Some(engine) => to_json(&engine.settings()),
            None => "null".to_owned(),
        }
    }

    fn publish(&mut self) {
        let text = self.state_text();
        if self.stateJson.to_string() != text {
            self.stateJson = QString::from(text.as_str());
            self.stateChanged();
        }
    }

    fn acceptNotice(&mut self) {
        if let Some(engine) = &self.engine {
            engine.accept_notice();
        }
        self.publish();
    }

    fn setSetting(&mut self, key: QString, value_json: QString) -> bool {
        let Some(engine) = self.engine.clone() else {
            return false;
        };
        let Ok(value) = serde_json::from_str(&value_json.to_string()) else {
            return false;
        };
        if !engine.set_setting(&key.to_string(), value) {
            return false;
        }
        self.settingsJson = QString::from(self.settings_text().as_str());
        self.settingsChanged();
        self.publish();
        true
    }

    fn search(&self, query: QString) -> QString {
        let hits = match &self.engine {
            Some(engine) => engine.search(&query.to_string()),
            None => Vec::new(),
        };
        QString::from(to_json(&hits).as_str())
    }

    fn addPlace(&mut self, name: QString, district: QString, km: i32) -> QString {
        let (Some(engine), Some(km)) = (self.engine.clone(), area_km(km)) else {
            return QString::from(add_error_key(AddError::Unknown));
        };
        let result = engine.add_place(&name.to_string(), &district.to_string(), km);
        self.publish();
        QString::from(result.err().map(add_error_key).unwrap_or_default())
    }

    fn setArea(&mut self, id: i32, km: i32) -> bool {
        let (Some(engine), Some(id), Some(km)) = (self.engine.clone(), place_id(id), area_km(km)) else {
            return false;
        };
        let changed = engine.set_area(id, km);
        self.publish();
        changed
    }

    fn removePlace(&mut self, id: i32) -> bool {
        let (Some(engine), Some(id)) = (self.engine.clone(), place_id(id)) else {
            return false;
        };
        let removed = engine.remove_place(id);
        self.publish();
        removed
    }

    fn setPosition(&mut self, lat: f64, lon: f64) -> bool {
        let Some(engine) = self.engine.clone() else {
            return false;
        };
        let taken = engine.set_position(lat, lon, now_ms());
        self.publish();
        taken
    }

    fn requestCheck(&self) {
        self.checkRequested();
    }

    fn reload(&mut self) {
        self.publish();
    }

    fn check(&mut self) {
        let Some(engine) = self.engine.clone() else { return };
        if self.checking {
            return;
        }
        self.checking = true;
        self.checkingChanged();
        let me = QPointer::from(&*self);
        let done = qmetaobject::queued_callback(move |notices: Vec<Notice>| {
            let Some(app) = me.as_pinned() else { return };
            let mut app = app.borrow_mut();
            app.checking = false;
            app.checkingChanged();
            app.publish();
            app.checked(QString::from(to_json(&notices).as_str()));
        });
        std::thread::spawn(move || done(engine.refresh(now_ms())));
    }

    fn warningJson(&self, id: QString) -> QString {
        let shown = self
            .engine
            .as_ref()
            .and_then(|engine| engine.warning(&id.to_string()));
        QString::from(to_json(&shown).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn areas_are_whole_district_or_up_to_the_limit() {
        assert_eq!(area_km(0), Some(0));
        assert_eq!(area_km(5), Some(5));
        assert_eq!(area_km(i32::from(MAX_KM)), Some(MAX_KM));
        assert_eq!(area_km(i32::from(MAX_KM) + 1), None);
        assert_eq!(area_km(-1), None);
        assert_eq!(area_km(300), None);
    }

    #[test]
    fn saved_places_have_positive_ids() {
        assert_eq!(place_id(1), Some(1));
        assert_eq!(place_id(0), None);
        assert_eq!(place_id(-3), None);
    }

    #[test]
    fn add_errors_have_keys() {
        assert_eq!(add_error_key(AddError::Unknown), "unknown");
        assert_eq!(add_error_key(AddError::Duplicate), "duplicate");
        assert_eq!(add_error_key(AddError::NotOffered), "notOffered");
    }

    #[test]
    fn json_text_of_values() {
        assert_eq!(to_json(&vec![1, 2]), "[1,2]");
        assert_eq!(to_json(&Option::<u8>::None), "null");
    }
}
