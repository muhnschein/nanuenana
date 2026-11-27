#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import base64
import json
import os
import sys
import urllib.request

HOST = "https://sonarcloud.io"
PAGE_SIZE = 500


def api_request(path, token):
    request = urllib.request.Request(HOST + "/api/" + path)
    if token:
        request.add_header("Authorization", "Basic " + base64.b64encode((token + ":").encode()).decode())
    return request


def fetch(path):
    with urllib.request.urlopen(api_request(path, os.environ.get("SONAR_READ_TOKEN")), timeout=60) as response:
        return json.load(response)


def location(item):
    return "%s:%s" % (item["component"].split(":", 1)[-1], item.get("line", "-"))


def fetch_all(get, path, list_name):
    items = []
    page = 1
    while True:
        data = get("%s&ps=%d&p=%d" % (path, PAGE_SIZE, page))
        items.extend(data.get(list_name, []))
        paging = data.get("paging", {})
        total = paging.get("total", data.get("total", len(items)))
        if page * PAGE_SIZE >= total or not data.get(list_name):
            return items, total
        page += 1


def report(get, key, scope, out):
    issues, issue_count = fetch_all(get, "issues/search?componentKeys=%s&%s&resolved=false" % (key, scope), "issues")
    hotspots, hotspot_count = fetch_all(get, "hotspots/search?projectKey=%s&%s&status=TO_REVIEW" % (key, scope),
                                        "hotspots")
    for i in issues:
        print("ISSUE   %-8s %-18s %s  %s" % (i.get("severity", ""), i["rule"], location(i), i["message"]), file=out)
    for h in hotspots:
        print("HOTSPOT %-8s %-18s %s  %s" % (h.get("vulnerabilityProbability", ""), h.get("ruleKey", ""),
                                             location(h), h["message"]), file=out)
    print("%d open issues, %d hotspots to review" % (issue_count, hotspot_count), file=out)
    return 1 if issue_count or hotspot_count else 0


def main():
    return report(fetch, os.environ["SONAR_PROJECT_KEY"], os.environ["SONAR_SCOPE"], sys.stdout)


if __name__ == "__main__":
    sys.exit(main())
