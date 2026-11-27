#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import json
import pathlib
import re
import sys
import urllib.error
import urllib.request

BASE = "https://warnung.bund.de/api31"
DISTRICT = re.compile(r"[0-9]{5}")
IDENTIFIER = re.compile(r"(?!\.)[A-Za-z0-9._-]{1,200}")
USAGE = "usage: replay.py record SET DISTRICT...\n       replay.py archive SET DISTRICT ARCHIVE_ID"


class Refused(Exception):
    pass


def http(url):
    try:
        with urllib.request.urlopen(url, timeout=30) as answer:
            return answer.read()
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise


def dashboard_name(district):
    if not DISTRICT.fullmatch(district):
        raise Refused(f"not a district code: {district}")
    return f"{district}0000000.json"


def checked(identifier):
    if not isinstance(identifier, str) or not IDENTIFIER.fullmatch(identifier):
        raise Refused(f"not a warning id: {identifier!r}")
    return identifier


def listed(body):
    try:
        entries = json.loads(body)
    except ValueError:
        return []
    if not isinstance(entries, list):
        return []
    found = []
    for entry in entries:
        identifier = entry.get("id") if isinstance(entry, dict) else None
        if isinstance(identifier, str) and IDENTIFIER.fullmatch(identifier) and identifier not in found:
            found.append(identifier)
    return found


def record(districts, fetch):
    files = {}
    for district in districts:
        name = dashboard_name(district)
        body = fetch(f"{BASE}/dashboard/{name}")
        if body is None:
            raise Refused(f"no dashboard for {district}")
        files[name] = body
        for identifier in listed(body):
            for ending in ("json", "geojson"):
                part = fetch(f"{BASE}/warnings/{identifier}.{ending}")
                if part is not None:
                    files[f"{identifier}.{ending}"] = part
    return files


def german(infos):
    for info in infos:
        if isinstance(info, dict) and info.get("language", "").lower().startswith("de"):
            return info
    return infos[0] if infos and isinstance(infos[0], dict) else {}


def entry_for(warning):
    identifier = checked(warning.get("identifier"))
    info = german(warning.get("info") or [])
    headline = info.get("headline") or ""
    return {
        "id": identifier,
        "payload": {
            "version": 1,
            "type": "ALERT",
            "id": identifier,
            "data": {
                "headline": headline,
                "provider": "MOWAS",
                "severity": info.get("severity") or "",
                "msgType": warning.get("msgType") or "",
            },
        },
        "i18nTitle": {"de": headline},
        "sent": warning.get("sent") or "",
    }


def archive(district, archive_id, fetch, listed_before=None):
    name = dashboard_name(district)
    body = fetch(f"{BASE}/archive.mowas/{checked(archive_id)}.json")
    if body is None:
        raise Refused(f"not in the archive: {archive_id}")
    try:
        warning = json.loads(body)
    except ValueError:
        warning = None
    if not isinstance(warning, dict):
        raise Refused(f"archive answer for {archive_id} is not a warning")
    entry = entry_for(warning)
    entries = [old for old in (listed_before or []) if not (isinstance(old, dict) and old.get("id") == entry["id"])]
    entries.append(entry)
    listing = (json.dumps(entries, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    return {name: listing, f"{entry['id']}.json": body}


def write(folder, files):
    folder.mkdir(parents=True, exist_ok=True)
    for name, body in sorted(files.items()):
        (folder / name).write_bytes(body)
        print(f"{folder / name}: {len(body)} bytes")


def before(folder, district):
    path = folder / dashboard_name(district)
    if not path.is_file():
        return []
    try:
        entries = json.loads(path.read_bytes())
    except ValueError:
        return []
    return entries if isinstance(entries, list) else []


def main(argv):
    try:
        if len(argv) >= 4 and argv[1] == "record":
            write(pathlib.Path(argv[2]), record(argv[3:], http))
            return 0
        if len(argv) == 5 and argv[1] == "archive":
            folder = pathlib.Path(argv[2])
            write(folder, archive(argv[3], argv[4], http, before(folder, argv[3])))
            return 0
    except Refused as refusal:
        print(f"replay.py: {refusal}", file=sys.stderr)
        return 1
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
