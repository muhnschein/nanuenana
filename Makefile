# SPDX-License-Identifier: GPL-3.0-or-later

CARGO ?= cargo
SDK_IMAGE ?= mirror.gcr.io/coderus/sailfishos-platform-sdk-aarch64:5.2.0.15@sha256:e5f7596d14502746b308c3dde36f65fc25d7f76c45a01acaed56a1f55b76f592
TARGET ?= SailfishOS-5.2.0.15-aarch64
export SDK_IMAGE TARGET

LUPDATE ?= lupdate
LUPDATE_ARGS = -silent -locations none -no-obsolete
TMP = $${TMPDIR:-/tmp}

.PHONY: translations translations-check comments-check check fmt-check clippy test coverage clippy-report mutants mutants-diff mutants-paths rpm check-rpm clean

check: fmt-check comments-check translations-check clippy test

comments-check:
	python3 tools/no_comments_test.py
	python3 tools/no_comments.py

fmt-check:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		QT_QPA_PLATFORM=offscreen $(CARGO) nextest run --workspace; \
	else \
		echo "test: cargo-nextest not installed, running cargo test (install-host-deps.sh nextest)"; \
		QT_QPA_PLATFORM=offscreen $(CARGO) test --workspace; \
	fi
	python3 tools/places_test.py
	python3 tools/untranslated_test.py
	python3 tools/replay_test.py
	python3 tools/ci/sonar_issues_test.py

PY_TESTS = tools/no_comments_test.py tools/places_test.py tools/untranslated_test.py tools/replay_test.py tools/ci/sonar_issues_test.py

coverage:
	mkdir -p coverage
	QT_QPA_PLATFORM=offscreen $(CARGO) llvm-cov --workspace --lcov --output-path coverage/lcov.info
	$(RM) coverage/python.data
	for t in $(PY_TESTS); do python3 -m coverage run --append --data-file=coverage/python.data --include='tools/*' $$t || exit 1; done
	python3 -m coverage xml --data-file=coverage/python.data -o coverage/python.xml

clippy-report:
	mkdir -p coverage
	$(CARGO) clippy --workspace --all-targets --message-format=json > coverage/clippy.json

MUTANTS_SHARD ?= 0/1
MUTANTS_DIFF ?= mutants.diff
MUTANTS_PATHS = crates/nanunana-core/src/alerts.rs crates/nanunana-core/src/bbk.rs \
	crates/nanunana-core/src/geo.rs crates/nanunana-core/src/model.rs \
	crates/nanunana-core/src/policy.rs crates/nanunana-core/src/settings.rs \
	crates/nanunana-core/src/text.rs crates/nanunana-core/src/view.rs \
	crates/nanunana-core/src/engine.rs crates/nanunana-core/src/gazetteer.rs \
	crates/nanunana-core/src/source.rs
MUTANTS_ARGS = --no-shuffle -j 2 --timeout 300 -p nanunana-core $(addprefix -f ,$(MUTANTS_PATHS))

mutants:
	$(CARGO) mutants $(MUTANTS_ARGS) --shard $(MUTANTS_SHARD)

mutants-diff:
	$(CARGO) mutants $(MUTANTS_ARGS) --in-diff $(MUTANTS_DIFF)

mutants-paths:
	@echo $(MUTANTS_PATHS)

rpm:
	./tools/ci/build-rpm.sh

check-rpm:
	./tools/ci/check-rpm.sh

translations:
	$(LUPDATE) $(LUPDATE_ARGS) -target-language en qml -ts translations/harbour-nanunana.ts
	python3 tools/english-plurals.py fill < translations/harbour-nanunana.ts > $(TMP)/nanunana-ts-filled.ts
	mv $(TMP)/nanunana-ts-filled.ts translations/harbour-nanunana.ts
	$(LUPDATE) $(LUPDATE_ARGS) -target-language de qml -ts translations/harbour-nanunana-de.ts

translations-check:
	cp translations/harbour-nanunana.ts $(TMP)/nanunana-ts-check.ts
	$(LUPDATE) $(LUPDATE_ARGS) -target-language en qml -ts $(TMP)/nanunana-ts-check.ts
	python3 tools/english-plurals.py fill < $(TMP)/nanunana-ts-check.ts > $(TMP)/nanunana-ts-checked.ts
	diff -u translations/harbour-nanunana.ts $(TMP)/nanunana-ts-checked.ts
	cp translations/harbour-nanunana-de.ts $(TMP)/nanunana-ts-de.ts
	$(LUPDATE) $(LUPDATE_ARGS) -target-language de qml -ts $(TMP)/nanunana-ts-de.ts
	diff -u translations/harbour-nanunana-de.ts $(TMP)/nanunana-ts-de.ts
	python3 tools/untranslated.py < translations/harbour-nanunana-de.ts

clean:
	$(CARGO) clean
	$(RM) -r coverage mutants.out RPMS prebuilt build-sdk-cross
