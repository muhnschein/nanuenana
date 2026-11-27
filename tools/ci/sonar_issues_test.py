#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
import io
import pathlib
import unittest
import urllib.parse

spec = importlib.util.spec_from_file_location("sonar_issues", pathlib.Path(__file__).with_name("sonar-issues.py"))
sonar = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sonar)


def issue(n):
    return {"severity": "MAJOR", "rule": "rust:S100", "component": "k:crates/a.rs", "line": n, "message": "m%d" % n}


def hotspot():
    return {"vulnerabilityProbability": "LOW", "ruleKey": "py:S5332", "component": "k:tools/a.py", "message": "h"}


class Server:
    def __init__(self, issues=(), hotspots=()):
        self.lists = {"issues": list(issues), "hotspots": list(hotspots)}
        self.asked = []

    def __call__(self, path):
        self.asked.append(path)
        api, query = path.split("?", 1)
        args = dict(urllib.parse.parse_qsl(query))
        name = api.split("/")[0]
        size, page = int(args["ps"]), int(args["p"])
        items = self.lists[name]
        return {name: items[(page - 1) * size:page * size], "paging": {"total": len(items)}}


def run(server, key="muhnschein_nanuenana", scope="pullRequest=7"):
    out = io.StringIO()
    return sonar.report(server, key, scope, out), out.getvalue(), server.asked


class Report(unittest.TestCase):
    def test_nothing_open_passes(self):
        code, text, _ = run(Server())
        self.assertEqual(code, 0)
        self.assertEqual(text, "0 open issues, 0 hotspots to review\n")

    def test_one_open_issue_fails_and_names_it(self):
        code, text, _ = run(Server(issues=[issue(3)]))
        self.assertEqual(code, 1)
        self.assertIn("ISSUE   MAJOR    rust:S100          crates/a.rs:3  m3\n", text)
        self.assertTrue(text.endswith("1 open issues, 0 hotspots to review\n"))

    def test_one_hotspot_fails_and_names_it(self):
        code, text, _ = run(Server(hotspots=[hotspot()]))
        self.assertEqual(code, 1)
        self.assertIn("HOTSPOT LOW      py:S5332           tools/a.py:-  h\n", text)

    def test_every_page_is_read(self):
        code, text, asked = run(Server(issues=[issue(n) for n in range(sonar.PAGE_SIZE + 1)]))
        self.assertEqual(code, 1)
        self.assertIn("crates/a.rs:%d " % sonar.PAGE_SIZE, text)
        self.assertEqual(len([p for p in asked if p.startswith("issues/")]), 2)

    def test_full_last_page_ends_without_another_request(self):
        _, _, asked = run(Server(issues=[issue(n) for n in range(sonar.PAGE_SIZE)]))
        self.assertEqual(len([p for p in asked if p.startswith("issues/")]), 1)

    def test_asks_only_for_unresolved_issues_and_unreviewed_hotspots_of_this_scope(self):
        _, _, asked = run(Server(), key="k1", scope="branch=main")
        self.assertEqual(asked, [
            "issues/search?componentKeys=k1&branch=main&resolved=false&ps=500&p=1",
            "hotspots/search?projectKey=k1&branch=main&status=TO_REVIEW&ps=500&p=1",
        ])


if __name__ == "__main__":
    unittest.main()
