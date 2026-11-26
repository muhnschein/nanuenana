#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import collections
import csv
import re
import sys

KINDS = ("Landkreis ", "Kreisfreie Stadt ", "Kreis ", "Stadtkreis ", "Regionalverband ", "Städteregion ", "Region ")
ORGANISATION = re.compile(
    r"GmbH|mbH|\bAG\b|\bKG\b|\beG\b|e\.\s?V\.|& Co|\bCo\.|Niederlassung|Agentur für|Universität|Klinik|Kranken"
    r"|[Ss]parkasse|Bezirksamt|\bARGE\b|Versicherung|Finanzamt|gericht\b|Verlag\b|\bAOK\b|\bIKK\b|direktion\b"
    r"|verwaltung\b|Bezirksverband|Staatsanwaltschaft|Brauerei"
)
EXONYMS = {"Nuremberg": "Nürnberg", "Munich": "München", "Cologne": "Köln", "Hanover": "Hannover"}
STATES = ("02000", "11000")


def district_name(district, names, codes):
    ranked = sorted(names.items(), key=lambda item: (not item[0].startswith(KINDS), -item[1], item[0]))
    name = ranked[0][0].split(",")[0].strip()
    name = EXONYMS.get(name, name)
    if district in STATES or name.startswith(KINDS) or name.startswith("Landeshauptstadt ") or "kreis" in name.lower():
        return name
    town = codes.get((name.split(" (")[0], district), set()) | codes.get((name, district), set())
    whole = set().union(*(found for (_, where), found in codes.items() if where == district))
    if 2 * len(town) > len(whole):
        return "Kreisfreie Stadt " + name
    return "Landkreis " + name


def postcodes(lines):
    return {line.split(",")[0] for line in lines if line[:5].isdigit()}


def build(rows, delivered):
    places = collections.defaultdict(lambda: {"lat": [], "lon": [], "codes": set()})
    names = collections.defaultdict(collections.Counter)
    for row in rows:
        district = row["community_code"]
        if len(district) != 5 or not district.isdigit():
            continue
        if row["zipcode"] not in delivered:
            continue
        place = " ".join(row["place"].split())
        if ORGANISATION.search(place):
            continue
        entry = places[(place, district)]
        entry["lat"].append(float(row["latitude"]))
        entry["lon"].append(float(row["longitude"]))
        entry["codes"].add(row["zipcode"])
        names[district][row["community"]] += 1
    place_lines = []
    for (place, district), entry in sorted(places.items()):
        lat = sum(entry["lat"]) / len(entry["lat"])
        lon = sum(entry["lon"]) / len(entry["lon"])
        codes = " ".join(sorted(entry["codes"]))
        place_lines.append(f"{place}\t{district}\t{lat:.4f}\t{lon:.4f}\t{codes}\n")
    codes = {key: entry["codes"] for key, entry in places.items()}
    district_lines = [
        f"{district}\t{district_name(district, found, codes)}\n" for district, found in sorted(names.items())
    ]
    return place_lines, district_lines


def main(argv):
    if len(argv) != 5:
        print("usage: places.py GEONAMES_POSTAL.csv DELIVERY_POSTCODES.csv PLACES.tsv DISTRICTS.tsv", file=sys.stderr)
        return 2
    with open(argv[2], encoding="utf-8") as source:
        delivered = postcodes(source)
    with open(argv[1], encoding="utf-8", newline="") as source:
        place_lines, district_lines = build(csv.DictReader(source), delivered)
    with open(argv[3], "w", encoding="utf-8") as out:
        out.writelines(place_lines)
    with open(argv[4], "w", encoding="utf-8") as out:
        out.writelines(district_lines)
    print(f"{len(place_lines)} places, {len(district_lines)} districts")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
