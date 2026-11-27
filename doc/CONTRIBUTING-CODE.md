# Code conventions

What Nanünana is: `PROJECT.md`. UI: design canvas in `../design` (one `.dc.html` artboard per screen; `design/STYLE.md` = Silica style kit).

## Toolchain

- Rust **1.95** everywhere (`rust-toolchain.toml`). 5.2 SDK target ships rustc 1.75, so RPM's binary is cross-built with host cargo against SDK's gcc, Qt, sysroot (`tools/ci/lift-sdk.sh`, `tools/ci/cross-build.sh`); `mb2` only packages it.
- `Cargo.lock` from current stable cargo honouring `.cargo/config.toml`'s `incompatible-rust-versions = "fallback"`: `cargo +stable update -p <crate>` / `cargo +stable generate-lockfile`. No crate newer than workspace `rust-version` allows.
- New dependencies: pure Rust preferred; native links only to Harbour's allowed libraries (`PROJECT.md` "Platform").

## Rules

- `make check` passes: `cargo fmt --check`, comment lint, catalogues, clippy with all of `clippy::all` denied, host tests.
- No `unwrap()`/`expect()`/`panic!` outside tests unless it cannot fire. Parsers of outside data never fail whole answer for one bad entry.
- `unsafe` only in `nanunana-qt` and `harbour-nanunana`.
- Logic lives in `nanunana-core`, without Qt, clock or socket: time is `now_ms` argument, network is `Fetch`. `nanunana-qt` only converts and forwards.
- Unit tests in `#[cfg(test)] mod tests`; behaviour across modules in `crates/nanunana-core/tests/`. Tests bite: CI runs `cargo-mutants` on core modules (changed lines per PR, all weekly; `make mutants-diff MUTANTS_DIFF=<file>` locally).
- SonarQube Cloud (`build.yml`, `sonar-project.properties`): token from GitHub OIDC (`sonarqube-auth-action`), no stored secret. Scan waits on quality gate (new-code coverage included); then `tools/ci/sonar-issues.py` fails on any open issue or unreviewed hotspot. Coverage: `make coverage` (`install-host-deps.sh coverage`). Finding → fix, or resolve on sonarcloud.io with reason.
- CodeQL (`codeql.yml`) scans shipped code only: no `cfg(test)`, no `crates/*/tests`, no `third_party` (`.github/codeql/codeql-config.yml`, `codeql-config-rust.yml`). `rust/access-invalid-pointer` is our copy (`.github/codeql/rust-queries`): silent on `#[derive(QObject)]` signal offsets (`&(*null).field`, reads no memory); its query test runs in same job. Finding → fix, or dismiss with reason.
- Never log position, place names or warning texts above `debug`.
- SPDX `// SPDX-License-Identifier: GPL-3.0-or-later` as line 1 of every source file (`#` in shell, Python, Makefile, YAML, TOML, spec; line 2 after a shebang). Third-party code keeps own licence and headers: `third_party/`; `NANUNANA.md` beside it says origin and changes.
- No code comments: AGENTS.md "Comments and docs".
- Strings: `qsTrId("nanunana-…")` with `//%` engineering English. Count in text = one plural string with `%n`, called `qsTrId(id, n)`. `make translations` refreshes `translations/harbour-nanunana.ts` (English, with singulars of new plurals: read printed pairs) and `harbour-nanunana-de.ts` (German: fill new entries by hand). `make check` fails on drift, on German gap, on placeholder mismatch.
- QML: Qt 5.6 JavaScript (no arrow functions, `let`, template strings). State arrives as JSON (`App.stateJson`, `App.settingsJson`); pages parse it, never keep own copy of truth.

## Layout

```
crates/nanunana-core     no Qt; model, bbk, geo, gazetteer, alerts, engine, settings, store, net, policy, text, view
crates/nanunana-qt       App singleton (qmetaobject), C++ glue (SailfishApp, translations, --qml-check)
crates/harbour-nanunana  binary (exported C main)
qml/                     pages/, components/, cover/, icons/
tests/qml                page tests for SDK target, canned answers in data/
tests/translations       catalogue test for SDK target
tools/                   comment lint, catalogue tools, place list generator, ci/
design/                  canvas boards, style kit
```

Shared core types: `Summary`/`Detail`/`Text`/`Level`/`Category`/`Kind` (model.rs), `Point`/`Shape` (geo.rs), `Settings`/`Threshold` (settings.rs), `Notice`/`NoticeKind` (alerts.rs), `View`/`PlaceView`/`WarningRow`/`WarningView`/`SearchHit` (view.rs), `Engine`/`Place` (engine.rs), `Fetch`/`Http`/`Canned` (net.rs).
