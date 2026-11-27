#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import io
import json
import re
import sys
import tarfile
import urllib.error
import urllib.request

BASE = "https://warnung.bund.de/api31"
DISTRICT = re.compile(r"\d{5}", re.ASCII)
IDENTIFIER = re.compile(r"(?!\.)[A-Za-z0-9._-]{1,200}")
USAGE = "usage: replay.py record DISTRICT... > SET.tar\n       replay.py archive DISTRICT ARCHIVE_ID > SET.tar"


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


def archive(district, archive_id, fetch):
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
    listing = (json.dumps([entry], ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    return {name: listing, f"{entry['id']}.json": body}


def packed(files):
    held = io.BytesIO()
    with tarfile.open(fileobj=held, mode="w") as tar:
        for name, body in sorted(files.items()):
            member = tarfile.TarInfo(name)
            member.size = len(body)
            tar.addfile(member, io.BytesIO(body))
    return held.getvalue()


def main(argv, fetch, out, errors):
    try:
        if len(argv) >= 3 and argv[1] == "record":
            files = record(argv[2:], fetch)
        elif len(argv) == 4 and argv[1] == "archive":
            files = archive(argv[2], argv[3], fetch)
        else:
            print(USAGE, file=errors)
            return 2
    except Refused as refusal:
        print(f"replay.py: {refusal}", file=errors)
        return 1
    out.write(packed(files))
    for name, body in sorted(files.items()):
        print(f"{name}: {len(body)} bytes", file=errors)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv, http, sys.stdout.buffer, sys.stderr))
