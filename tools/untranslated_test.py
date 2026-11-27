#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
import io
import pathlib
import unittest

spec = importlib.util.spec_from_file_location("untranslated", pathlib.Path(__file__).with_name("untranslated.py"))
untranslated = importlib.util.module_from_spec(spec)
spec.loader.exec_module(untranslated)


def message(name, source, translation, numerus=False):
    kind = ' numerus="yes"' if numerus else ""
    return f'<message id="{name}"{kind}>\n<source>{source}</source>\n{translation}\n</message>\n'


class Gaps(unittest.TestCase):
    def test_finished_messages_pass(self):
        text = message("a", "Until %1", "<translation>Bis %1</translation>")
        text += message("b", "%n places", "<translation><numerusform>%n Ort</numerusform><numerusform>%n Orte</numerusform></translation>", True)
        self.assertEqual(untranslated.gaps(text), [])

    def test_unfinished_and_empty_are_gaps(self):
        text = message("a", "Save", '<translation type="unfinished"></translation>')
        text += message("b", "Quit", "<translation> </translation>")
        text += message("c", "%n places", "<translation><numerusform>%n Ort</numerusform><numerusform></numerusform></translation>", True)
        text += message("d", "Gone", "")
        self.assertEqual(
            untranslated.gaps(text),
            ["a: not translated", "b: empty translation", "c: empty translation", "d: not translated"],
        )

    def test_placeholders_must_match(self):
        text = message("a", "%1 km of %2", "<translation>%1 km</translation>")
        text += message("b", "%n places", "<translation><numerusform>ein Ort</numerusform><numerusform>%n Orte</numerusform></translation>", True)
        text += message("c", "Plain", "<translation>Schlicht %1</translation>")
        found = untranslated.gaps(text)
        self.assertEqual([gap.split(":")[0] for gap in found], ["a", "b", "c"])
        self.assertIn("%1, %2", found[0])


class Run(unittest.TestCase):
    def test_a_finished_catalogue_passes_in_silence(self):
        errors = io.StringIO()
        text = message("a", "Until %1", "<translation>Bis %1</translation>")
        self.assertEqual(untranslated.main(io.StringIO(text), errors), 0)
        self.assertEqual(errors.getvalue(), "")

    def test_every_gap_is_named_and_fails_the_run(self):
        errors = io.StringIO()
        text = message("a", "Save", '<translation type="unfinished"></translation>')
        text += message("b", "Quit", "<translation> </translation>")
        self.assertEqual(untranslated.main(io.StringIO(text), errors), 1)
        self.assertEqual(errors.getvalue(), "untranslated: a: not translated\nuntranslated: b: empty translation\n")


if __name__ == "__main__":
    unittest.main()
