# AGENTS.md

How to work on Nanünana. What it is: `README.md`, `doc/PROJECT.md`. Read `doc/CONTRIBUTING-CODE.md` before first change. User's words in chat beat this file.

## Ground rules

- Don't know how to proceed → say "I don't know", stop, ask. Mark guesses as guesses. Unknown beats confident wrong.
- Bugs: find root cause, state it. Symptom fix ≠ fix.
- Measure twice, cut once. YAGNI: fight scope creep; meet user's intent minimally, realistically. Good return on cost; no gold-plating, no unasked refactors.
- Rule here fights task → say so loudly, get user's sign-off before breaking it.
- Options → lay them out, recommend one with reasons. Implement nothing until user clearly picks.
- Warning app: a missed or wrong warning can hurt someone. Never show "No warnings" for a place whose last check failed or is old. Never change, shorten or machine-translate an issuer's text. Fail open: doubt whether a warning concerns a place → show it.
- App is unofficial and says so: first start, home, every place, every warning, About. Never remove or weaken those lines; never imitate an official app's name, icon or look.
- Where offered: `crates/nanunana-core/src/policy.rs`. `BLOCKED` (Russian Federation, Belarus, Iran, North Korea, Syria; Russian-controlled territories outside Russia once a neighbouring country is supported) only by owner's word. New country → owner's go, its official source, its official-channels text.

## Done means

In this order; later step only once earlier ones hold:

1. Change implemented.
2. Tests added for it (regression test per bug).
3. Tests proven to bite: each fails with change reverted or mutated. Report how (e.g. "22/22 mutations caught"); name what headless tests can't prove. Change touches a mutated file (`make -s mutants-paths`) → `make mutants-diff MUTANTS_DIFF=<diff>` clean.
4. `make check` green locally.
5. PR CI green: CI (host checks, mutants on changed lines, RPM with Harbour check), SonarQube (quality gate, new-code coverage, zero open issues or hotspots: `tools/ci/sonar-issues.py`), CodeQL.
6. Device: CI job "Sailfish OS 5.2 aarch64 RPM" builds it on every push; run that RPM on test phone (see "Device tests"); give user the `harbour-nanunana-aarch64` artifact link of the green run and the evidence.
7. With RPM: outline briefly what merging this PR risks and how you mitigated it.

PR opens as draft on first push. Mark ready for review only once steps 1–7 hold and their evidence is posted (visual: project thread; text: PR body).

No phone run when change can't show on phone: docs, CI, build scripts (RPM contents unchanged), tests only. Steps 6–7 skipped; PR says why.

User tests RPM on phone; that test judges feel, lag, transitions. Say exactly what to check on device.

## Device tests

- Page tests (`tests/qml/tst_*.qml`) and OS file lookups (Silica QML, theme icons, sailjail profiles): SDK container, never phone.
- Test phone: on the local SFOS test host via the harness's `local/run.sh` (`muhnschein-sfos-harness`, see its `local/README.md`); Nanünana's caller notes: its `apps/nanunana.md`. Its AGENTS.md rules every phone run (baseline, no blind taps, reboot, language, `clean_slate`); helpers: its README, `scripts/lib.sh`. Never run fork or others' PR code there: any run can root phone.
- PR build onto phone: `install_artifact muhnschein/nanuenana RUN_ID harbour-nanunana`, RUN_ID of this PR's green CI run.
- Every run starts from named state; `state.txt` records it plus RPM name and sha256:
  - `clean`: `reset`: quit app, drop sailjail grant, `rpm -e`, delete `~/.local/share/nanunana/nanunana`, `~/.config/nanunana/nanunana`, `~/.cache/nanunana/nanunana`; install.
  - `fixture:NAME`: `clean`, then restore data snapshot saved with `fixture_save`.
  - `upgrade:FROM`: release FROM plus fixture, then install build over it. Required when change touches stored data (`state.json`, `cache.json`) or settings.
- Warnings on phone are live data: say which were listed when, never expect a fixed one. No live warning for the case → page tests with canned data prove it; say so.
- Before/after: same start state, same steps, one `local/run.sh` run (both builds, one `launch` each).
- Build already passed same steps → cite that run, no rerun.
- Privacy: user stays muhnschein, nothing more. Harness output leaves the test host only scrubbed, through its `local/publish.sh` (no location, addresses, device or network ids; its AGENTS.md "Privacy"). Evidence only from those files; never type identifiers in. Location tests: harness fake places only, never real fix. Saved places in tests: large cities, never anyone's home.
- Harness proves function and frame timing. Feel, lag, transitions stay user's call on device.

## Local checks

- Host deps (Ubuntu 24.04): `./tools/ci/install-host-deps.sh [nextest] [mutants] [coverage] [cross]`: Qt 5 dev files, `lupdate`, pinned Rust 1.95; extras = pinned cargo-nextest (`make test` runs 32 tests at once; without it `cargo test`), cargo-mutants, cargo-llvm-cov and python3-coverage (`make coverage`), `libc6-i386`.
- `make check` = rustfmt, comment lint, catalogue drift and German gaps, clippy (`clippy::all` denied), host tests (Rust, tool tests). No network: tests use canned answers (`crates/nanunana-core/tests/fixtures`, `tests/qml/data`).
- Page tests (`tests/qml/tst_*.qml`, real Silica, Qt 5.6) run only on SDK target: `make rpm && make check-rpm` (docker, `libc6-i386`; also Harbour validator, linked libs, sandbox permissions, installability). QML change → run it, or say only CI did.
- Missing tool (docker, cargo-mutants) → step skipped locally. Say which; CI runs all.
- Local failure → rerun on untouched `main` before calling it environment.
- Iterating → run only what change touches: `cargo test -p nanunana-core <filter>`. Before every push: full `make check`. CI runs everything (docs-only PR or push to main: no build or test jobs, CodeQL actions only; `tools/ci/changes.sh`).
- Gates catch device-only breakage (Qt 5.6 QML, Harbour rules of `doc/PROJECT.md` "Platform", linked libs, validator). Gate red → fix code, never loosen gate. New validator warning only if unavoidable, listed with reason in `doc/harbour-warnings.md`.
- Mutant exclusion (`.cargo/mutants.toml`) only with proof it changes no observable behaviour, in commit body and PR.
- New dependency must earn its keep: well maintained, widely used, more than someone's pet project, builds at Rust 1.95, pure Rust preferred, native links only to Harbour's allowed libraries (`doc/PROJECT.md`). Say why in PR. `Cargo.lock` via `cargo +stable update -p <crate>` (CONTRIBUTING-CODE.md "Toolchain").
- Place list (`crates/nanunana-core/data`) is generated: `tools/places.py`, sources in `doc/PROJECT.md` "Places". Never edit by hand.

## New tests

Every added or changed test, before push:

- Honest: fails when behaviour its name claims breaks; "Done means" step 3 proves it. No assert that cannot fail: tautology, check of own setup, assert in branch or loop that may not run, discarded result, timeout read as answer.
- Waits on signal or condition only. No sleep, wall-clock assert. Timeout = hang guard: seconds for work of milliseconds. Fixed wait only for "nothing happens", where no event exists; test name says so.
- Parallel-safe: passes alone and among 32 others (`make test`). Own tempdir; no fixed port or path, shared HOME/XDG, env or cwd change shared with other tests, global state across tests.
- Time is an argument (`now_ms`), never the clock. Network is `Fetch`, never a socket.

## UI and words

- Silica stock components only, built like existing pages; idioms in `design/STYLE.md` (pulley, no push-up menu, remorse, dialog pages). Jolla app has same feature → match it.
- "Design first" / "no PR yet" → artboard on design canvas (`design/canvas/project/*.dc.html`, kit `design/STYLE.md`), wait for approval, then code.
- Owner's design rulings: no custom place names (place name = location name, shown once); About and warning language reachable only from Settings; no footnotes or explaining subtitles where the control says it; keep it simple.
- Every string `qsTrId()` with `//%` engineering English. Changed strings → `make translations`, then fill `translations/harbour-nanunana-de.ts` (`make check` fails on drift or German gap). A count in a text → one `%n` plural string, never "(s)"; `make translations` adds its English singular (read the printed pairs).
- Issuer's texts (headline, description, instruction, area, contact) shown as plain text, unchanged, in issuer's language or issuer's own translation.
- Target page tests have no finger, no phone: looks and gestures unproven until device test. Say so.
- Phone = battery: no always-running animations, timers, polling. One wake-up per check interval (`BackgroundJob`), position asked once per check and only with the setting on, nothing else in background.

## Comments and docs

- Caveman-full = drop articles, filler, pleasantries, hedging. Fragments OK, short words. Identifiers, paths, code, errors exact. E.g. "`Cargo.lock` from stable cargo with fallback resolver: no crate newer than `rust-version` allows."
- No code comments. None: no why, no issue reference, no TODO, no doc comment, no commented-out code; in Rust, QML, JS, shell, Python, Makefile, YAML, TOML, spec, desktop file. Names and tests say what code does. `make check` runs `tools/no_comments.py`.
- No workarounds for this rule, no write-ups of code workarounds: no comment archive, no notes or doc file standing in for comments, no comment text hidden in strings or names. Only an absolutely critical fact (code breaks or regresses without it) gets written down: project memory, one short sentence each, nothing more.
- Kept, because a tool reads them: shebang; SPDX line 1 (line 2 after shebang); `//%` lupdate source text; `/// # Safety` section right above a `pub unsafe fn` (clippy `missing_safety_doc`). Not checked: `third_party/`, CodeQL query copy (licences keep their notices), fixtures, `doc/`, `design/`, Markdown.
- `doc/*.md`, this file: caveman-full. What is true now and why; keep constraints. No changelog, bug report, history, story (git's job). Existing prose: rewrite what you touch, not more.
- No specs: code and tests say what it does.
- `README.md`, `CHANGELOG.md`, release notes, store and forum posts: plain prose for users, as short and to the point as possible.
- User-visible change → line in `CHANGELOG.md` under `## Unreleased` (file or section missing → create it), same PR.
- No plans, research notes, scratch files in repo.

## Git and PRs

- One branch, one PR per task. Stacked PR only when asked.
- Unrelated finding → tell user; separate PR or issue, not this PR.
- Commit and PR titles: Conventional Commits, plain language, say outcome: `fix(alerts): an ended warning no longer alerts twice`. Types `feat` `fix` `perf` `refactor` `test` `docs` `ci` `build` `chore`; scope = app area or subsystem (`places`, `warnings`, `alerts`, `location`, `settings`, `cover`, `rpm`, …). Body says why, plain prose; brief, to the point.
- PR body: the problem in a sentence or two, then how you fixed it.
- Final diff: no debug logging, no leftovers.
- Evidence shown, not just described (visual: files in project thread; text: PR body):
  - UI → before/after device screenshots.
  - Transitions, motion, feel, lag → before/after GIF or mp4.
  - Non-visual proof (tests, mutations, logs) → text.
- Evidence never on GitHub: no `evidence/` branches, no uploads to PR, no assets in `main` or PR branch.
- Flaky test → find and fix cause. Never add retries: pass on second try = defect. Never re-run CI until green and call it done. Never skip test or loosen assert. One re-run OK only for infra failure before any test ran (runner lost, apt mirror timeout); say so.
- Merge only on user's explicit go. Merge commit, not squash/rebase.
- Before asking for merge go: `main` moved since PR's last green run → merge `main` in, CI green again.

## Releases

- Only on user's go. None cut yet, no tags. Latest `main` commit green first, Sonar included: check main, not just PR. `main` commit without runs (push event lost) → start CI, Build, CodeQL on `main` by hand (`workflow_dispatch`).
- Strict semver: nothing breaking → minor.
- Harbour upload stays with user.

## Subagents

- Delegate only with tight, written scope.
- Cheap-model output gets strong-model review: cheap models dropped facts, inverted conditions.

## Reporting

- Replies to user: caveman-full (see Comments and docs).
- Lead with result. Say what was verified, what wasn't, what is guess.
- Non-green step → name it and why. Never hide.
- Bug report → search GitHub issues first, link match.
