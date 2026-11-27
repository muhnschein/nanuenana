#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import contextlib
import importlib.util
import io
import json
import pathlib
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("replay", pathlib.Path(__file__).with_name("replay.py"))
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

BASE = replay.BASE


class Net:
    def __init__(self, answers):
        self.answers = answers
        self.asked = []

    def __call__(self, url):
        self.asked.append(url)
        return self.answers.get(url)


class Anything:
    def __call__(self, url):
        return b"[]"


def listing(*identifiers):
    return json.dumps([{"id": identifier, "payload": {"version": 3}} for identifier in identifiers]).encode()


class Record(unittest.TestCase):
    def test_answers_are_kept_byte_for_byte_under_the_service_file_names(self):
        dashboard = b'[ {"id": "mow.A-1", "x": "\xc3\xa4"},\n {"id":"dwd.B_2"} ]'
        net = Net({
            f"{BASE}/dashboard/091620000000.json": dashboard,
            f"{BASE}/warnings/mow.A-1.json": b'{"identifier" : "mow.A-1"}',
            f"{BASE}/warnings/mow.A-1.geojson": b'{"type":"FeatureCollection"}',
            f"{BASE}/warnings/dwd.B_2.json": b"{}",
        })
        files = replay.record(["09162"], net)
        self.assertEqual(files, {
            "091620000000.json": dashboard,
            "mow.A-1.json": b'{"identifier" : "mow.A-1"}',
            "mow.A-1.geojson": b'{"type":"FeatureCollection"}',
            "dwd.B_2.json": b"{}",
        })
        self.assertEqual(net.asked, [
            f"{BASE}/dashboard/091620000000.json",
            f"{BASE}/warnings/mow.A-1.json",
            f"{BASE}/warnings/mow.A-1.geojson",
            f"{BASE}/warnings/dwd.B_2.json",
            f"{BASE}/warnings/dwd.B_2.geojson",
        ])

    def test_several_districts_and_an_empty_one(self):
        net = Net({
            f"{BASE}/dashboard/091620000000.json": b"[]",
            f"{BASE}/dashboard/020000000000.json": listing("pol.X"),
            f"{BASE}/warnings/pol.X.json": b"{}",
        })
        files = replay.record(["09162", "02000"], net)
        self.assertEqual(sorted(files), ["020000000000.json", "091620000000.json", "pol.X.json"])

    def test_ids_that_could_leave_the_folder_are_not_asked_for(self):
        body = json.dumps([{"id": "../x"}, {"id": ".hidden"}, {"id": "a/b"}, {"id": 7}, "text", {"id": "ok.1"}, {"id": "ok.1"}]).encode()
        net = Net({f"{BASE}/dashboard/091620000000.json": body})
        files = replay.record(["09162"], net)
        self.assertEqual(sorted(files), ["091620000000.json"])
        self.assertEqual(net.asked[1:], [f"{BASE}/warnings/ok.1.json", f"{BASE}/warnings/ok.1.geojson"])

    def test_a_dashboard_that_is_no_list_is_kept_and_asks_nothing_more(self):
        for body in (b"not json", b'{"id": "mow.A"}', b"7", b"null"):
            net = Net({f"{BASE}/dashboard/091620000000.json": body})
            self.assertEqual(replay.record(["09162"], net), {"091620000000.json": body})
            self.assertEqual(len(net.asked), 1)

    def test_a_bad_district_or_a_missing_dashboard_is_refused(self):
        for district in ("9162", "091620", "0916a", "../09162", ""):
            with self.assertRaises(replay.Refused, msg=district):
                replay.record([district], Anything())
        with self.assertRaises(replay.Refused):
            replay.record(["09162"], Net({}))


WARNING = {
    "identifier": "mow.DE-NI-OL-W015-20230121-001",
    "sent": "2023-01-21T11:26:59+01:00",
    "msgType": "Alert",
    "info": [
        {"language": "en", "headline": "Fire", "severity": "Minor"},
        {"language": "DE", "headline": "Großbrand", "severity": "Severe"},
    ],
}


class Archive(unittest.TestCase):
    def test_the_archived_warning_is_kept_as_it_is_and_listed_for_the_district(self):
        body = json.dumps(WARNING, ensure_ascii=False, indent=1).encode("utf-8")
        net = Net({f"{BASE}/archive.mowas/mow.DE-NI-OL-W015-20230121-001_20230121112659.json": body})
        files = replay.archive("03403", "mow.DE-NI-OL-W015-20230121-001_20230121112659", net)
        self.assertEqual(sorted(files), ["034030000000.json", "mow.DE-NI-OL-W015-20230121-001.json"])
        self.assertEqual(files["mow.DE-NI-OL-W015-20230121-001.json"], body)
        self.assertEqual(json.loads(files["034030000000.json"]), [{
            "id": "mow.DE-NI-OL-W015-20230121-001",
            "payload": {
                "version": 1,
                "type": "ALERT",
                "id": "mow.DE-NI-OL-W015-20230121-001",
                "data": {"headline": "Großbrand", "provider": "MOWAS", "severity": "Severe", "msgType": "Alert"},
            },
            "i18nTitle": {"de": "Großbrand"},
            "sent": "2023-01-21T11:26:59+01:00",
        }])
        self.assertIn("Großbrand".encode("utf-8"), files["034030000000.json"])

    def test_it_joins_what_the_set_already_lists_and_replaces_its_own_entry(self):
        body = json.dumps(WARNING).encode()
        net = Net({f"{BASE}/archive.mowas/x_1.json": body})
        old = [{"id": "pol.X"}, {"id": "mow.DE-NI-OL-W015-20230121-001", "old": True}, "odd"]
        files = replay.archive("03403", "x_1", net, old)
        entries = json.loads(files["034030000000.json"])
        self.assertEqual([e["id"] if isinstance(e, dict) else e for e in entries], ["pol.X", "odd", "mow.DE-NI-OL-W015-20230121-001"])
        self.assertNotIn("old", entries[2])

    def test_without_a_german_text_the_first_one_is_listed(self):
        warning = dict(WARNING, info=[{"language": "en", "headline": "Fire", "severity": "Minor"}])
        net = Net({f"{BASE}/archive.mowas/x_1.json": json.dumps(warning).encode()})
        entry = json.loads(replay.archive("03403", "x_1", net)["034030000000.json"])[0]
        self.assertEqual((entry["i18nTitle"]["de"], entry["payload"]["data"]["severity"]), ("Fire", "Minor"))

    def test_missing_fields_stay_empty(self):
        net = Net({f"{BASE}/archive.mowas/x_1.json": b'{"identifier": "mow.A"}'})
        entry = json.loads(replay.archive("03403", "x_1", net)["034030000000.json"])[0]
        self.assertEqual(entry["payload"]["data"], {"headline": "", "provider": "MOWAS", "severity": "", "msgType": ""})
        self.assertEqual(entry["sent"], "")

    def test_what_is_not_an_archived_warning_is_refused(self):
        cases = {
            "missing": {},
            "not json": {f"{BASE}/archive.mowas/x_1.json": b"<html>"},
            "a list": {f"{BASE}/archive.mowas/x_1.json": b"[]"},
            "no identifier": {f"{BASE}/archive.mowas/x_1.json": b"{}"},
            "bad identifier": {f"{BASE}/archive.mowas/x_1.json": b'{"identifier": "../x"}'},
        }
        for name, answers in cases.items():
            with self.assertRaises(replay.Refused, msg=name):
                replay.archive("03403", "x_1", Net(answers))
        warning = json.dumps(WARNING).encode()
        for archive_id in ("../x", ".x", "a/b", ""):
            net = Net({f"{BASE}/archive.mowas/{archive_id}.json": warning})
            with self.assertRaises(replay.Refused, msg=archive_id):
                replay.archive("03403", archive_id, net)
            self.assertEqual(net.asked, [])
        net = Net({f"{BASE}/archive.mowas/x_1.json": warning})
        with self.assertRaises(replay.Refused):
            replay.archive("3403", "x_1", net)
        self.assertEqual(net.asked, [])


class Files(unittest.TestCase):
    def test_a_set_is_written_and_read_back_for_the_next_archive_call(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = pathlib.Path(tmp) / "sets" / "one"
            said = io.StringIO()
            with contextlib.redirect_stdout(said):
                replay.write(folder, {"034030000000.json": b'[{"id": "pol.X"}]', "pol.X.json": b"{}"})
            self.assertEqual(said.getvalue().count("bytes"), 2)
            self.assertEqual((folder / "pol.X.json").read_bytes(), b"{}")
            self.assertEqual(replay.before(folder, "03403"), [{"id": "pol.X"}])
            self.assertEqual(replay.before(folder, "09162"), [])
            (folder / "091620000000.json").write_bytes(b"{}")
            self.assertEqual(replay.before(folder, "09162"), [])
            (folder / "091620000000.json").write_bytes(b"broken")
            self.assertEqual(replay.before(folder, "09162"), [])

    def test_wrong_use_prints_the_usage(self):
        said = io.StringIO()
        with contextlib.redirect_stderr(said):
            self.assertEqual(replay.main(["replay.py"]), 2)
            self.assertEqual(replay.main(["replay.py", "record", "set"]), 2)
            self.assertEqual(replay.main(["replay.py", "archive", "set", "03403"]), 2)
            self.assertEqual(said.getvalue().count("usage:"), 3)
            self.assertEqual(replay.main(["replay.py", "record", "set", "x"]), 1)
        self.assertIn("not a district code: x", said.getvalue())


if __name__ == "__main__":
    unittest.main()
