#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_file_location("places", pathlib.Path(__file__).with_name("places.py"))
places = importlib.util.module_from_spec(spec)
spec.loader.exec_module(places)


def row(place, code, community, zipcode, lat="50.0", lon="8.0"):
    return {"place": place, "community_code": code, "community": community, "zipcode": zipcode, "latitude": lat, "longitude": lon}


class Build(unittest.TestCase):
    def test_places_merge_postcodes_and_average_their_position(self):
        rows = [
            row("Köln", "05315", "Kreisfreie Stadt Köln", "50667", "50.0", "7.0"),
            row("Köln", "05315", "Kreisfreie Stadt Köln", "50668", "51.0", "6.0"),
        ]
        lines, districts = places.build(rows, {"50667", "50668"})
        self.assertEqual(lines, ["Köln\t05315\t50.5000\t6.5000\t50667 50668\n"])
        self.assertEqual(districts, ["05315\tKreisfreie Stadt Köln\n"])

    def test_bulk_recipient_postcodes_and_organisations_are_left_out(self):
        rows = [
            row("Augsburg", "09761", "Kreisfreie Stadt Augsburg", "86150"),
            row("Dr. Grandel GmbH", "09761", "Kreisfreie Stadt Augsburg", "86147"),
            row("Karstadt AG", "09761", "Kreisfreie Stadt Augsburg", "86150"),
            row("Stadtsparkasse", "09761", "Kreisfreie Stadt Augsburg", "86150"),
        ]
        lines, _ = places.build(rows, {"86150"})
        self.assertEqual([line.split("\t")[0] for line in lines], ["Augsburg"])

    def test_rows_without_a_district_code_are_left_out(self):
        rows = [row("Nirgends", "", "", "00001"), row("Halb", "0531", "", "00002")]
        self.assertEqual(places.build(rows, {"00001", "00002"}), ([], []))

    def test_bare_district_names_get_their_kind(self):
        rows = [
            row("Saarlouis", "10044", "Saarlouis", "66740"),
            row("Dillingen", "10044", "Saarlouis", "66763"),
            row("Rehlingen-Siersburg", "10044", "Saarlouis", "66780"),
            row("Lübeck", "01003", "Lübeck", "23552"),
            row("Nürnberg", "09564", "Nuremberg", "90402"),
            row("Hamburg", "02000", "Hamburg", "20095"),
            row("Bad Homburg", "06434", "Hochtaunuskreis", "61348"),
        ]
        _, districts = places.build(rows, {r["zipcode"] for r in rows})
        self.assertEqual(
            districts,
            [
                "01003\tKreisfreie Stadt Lübeck\n",
                "02000\tHamburg\n",
                "06434\tHochtaunuskreis\n",
                "09564\tKreisfreie Stadt Nürnberg\n",
                "10044\tLandkreis Saarlouis\n",
            ],
        )

    def test_the_most_common_kind_name_wins(self):
        rows = [
            row("A", "08335", "Konstanz, Landkreis", "78462"),
            row("B", "08335", "Landkreis Konstanz", "78464"),
            row("C", "08335", "Landkreis Konstanz", "78465"),
        ]
        _, districts = places.build(rows, {"78462", "78464", "78465"})
        self.assertEqual(districts, ["08335\tLandkreis Konstanz\n"])


class Postcodes(unittest.TestCase):
    def test_only_lines_starting_with_a_postcode_count(self):
        self.assertEqual(places.postcodes([",lat,lng\n", "01067,51.0,13.7\n", "x\n", "86150,48.3,10.9\n"]), {"01067", "86150"})


if __name__ == "__main__":
    unittest.main()
