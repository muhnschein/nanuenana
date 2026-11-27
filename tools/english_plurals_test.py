#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
import io
import pathlib
import sys
import unittest
import unittest.mock

spec = importlib.util.spec_from_file_location("english_plurals", pathlib.Path(__file__).with_name("english-plurals.py"))
plurals = importlib.util.module_from_spec(spec)
spec.loader.exec_module(plurals)

HEAD = '<?xml version="1.0" encoding="utf-8"?>\n<!DOCTYPE TS>\n<TS version="2.1" language="en">\n<context>\n    <name></name>\n'
TAIL = "</context>\n</TS>\n"


def plain(name, source, translation='<translation type="unfinished"></translation>'):
    return f'    <message id="{name}">\n        <source>{source}</source>\n        {translation}\n    </message>\n'


def counted(name, source, forms=None, extra=""):
    if forms is None:
        body = '        <translation type="unfinished">\n            <numerusform></numerusform>\n        </translation>\n'
    else:
        lines = "".join(f"            <numerusform>{form}</numerusform>\n" for form in forms)
        body = f"        <translation>\n{lines}        </translation>\n"
    return f'    <message id="{name}" numerus="yes">\n        <source>{source}</source>\n{extra}{body}    </message>\n'


class Singular(unittest.TestCase):
    def test_nouns_lose_their_plural_ending(self):
        pairs = {"places": "place", "entries": "entry", "classes": "class", "boxes": "box", "warnings": "warning"}
        for many, single in pairs.items():
            self.assertEqual(plurals.one(many), single)

    def test_words_that_do_not_change_stay(self):
        for word in ("more", "at", "selected", "missing"):
            self.assertEqual(plurals.one(word), word)

    def test_a_noun_it_cannot_tell_is_refused(self):
        for word in ("matches", "dishes", "bus", "glass", "axis", "sheep"):
            with self.assertRaises(ValueError, msg=word):
                plurals.one(word)

    def test_the_verb_after_the_noun_follows(self):
        self.assertEqual(plurals.singular("%n places are listed"), "%n place is listed")
        self.assertEqual(plurals.singular("%n warnings haven&apos;t ended"), "%n warning hasn&apos;t ended")
        self.assertEqual(plurals.singular("There were %n places that have ended"), "There were %n place that has ended")
        self.assertEqual(plurals.singular("%n places were listed"), "%n place was listed")
        self.assertEqual(plurals.singular("%n places aren&apos;t listed"), "%n place isn&apos;t listed")
        self.assertEqual(plurals.singular("%n places weren&apos;t listed"), "%n place wasn&apos;t listed")
        self.assertEqual(plurals.singular("%n places don&apos;t count"), "%n place doesn&apos;t count")
        self.assertEqual(plurals.singular("%n places share hardware"), "%n place share hardware")
        self.assertEqual(plurals.singular("+ %n more"), "+ %n more")
        self.assertEqual(plurals.singular("No count here are"), "No count here are")


class Fill(unittest.TestCase):
    def test_an_empty_plural_gets_singular_and_plural(self):
        text = HEAD + plain("a", "Save") + counted("b", "%n places") + TAIL
        out, made, refused = plurals.fill(text)
        self.assertEqual(out, HEAD + plain("a", "Save") + counted("b", "%n places", ["%n place", "%n places"]) + TAIL)
        self.assertEqual(made, [("b", ["%n place", "%n places"])])
        self.assertEqual(refused, [])

    def test_a_filled_plural_is_left_alone_and_a_stale_one_is_redone(self):
        kept = counted("a", "%n places", ["one place", "%n places"])
        stale = counted("b", "%n warnings", ["%n place", "%n places"], "        <oldsource>%n places</oldsource>\n")
        half = counted("c", "%n boxes", ["", "%n boxes"])
        long = counted("d", "%n entries", ["%n entry", "%n entries", "%n entries"])
        out, made, refused = plurals.fill(HEAD + kept + stale + half + long + TAIL)
        self.assertEqual(out, HEAD + kept + counted("b", "%n warnings", ["%n warning", "%n warnings"])
                         + counted("c", "%n boxes", ["%n box", "%n boxes"])
                         + counted("d", "%n entries", ["%n entry", "%n entries"]) + TAIL)
        self.assertEqual([name for name, _ in made], ["b", "c", "d"])
        self.assertEqual(refused, [])

    def test_texts_it_must_not_guess_are_refused_and_left_as_they_are(self):
        text = (HEAD + plain("a", "%n places") + counted("b", "%n place(s)") + counted("c", "%n match(es)")
                + counted("d", "%n matches") + TAIL)
        out, made, refused = plurals.fill(text)
        self.assertEqual(out, text)
        self.assertEqual(made, [])
        self.assertEqual(refused, [
            "a: has %n but qsTrId gets no count",
            "b: write a plain plural, not (s)",
            "c: write a plain plural, not (s)",
            'd: cannot tell the singular of "matches": write it by hand',
        ])

    def test_the_catalogue_is_marked_english(self):
        bare = HEAD.replace(' language="en"', "")
        out, _, _ = plurals.fill(bare + TAIL)
        self.assertEqual(out, HEAD + TAIL)
        german = HEAD.replace('"en"', '"de"') + TAIL
        with self.assertRaisesRegex(ValueError, 'must be language="en"'):
            plurals.fill(german)
        with self.assertRaisesRegex(ValueError, "not a TS catalogue"):
            plurals.fill("<xml/>")


class Run(unittest.TestCase):
    def run_with(self, text, *words):
        streams = [io.TextIOWrapper(io.BytesIO(text.encode("utf-8")), encoding="ascii")]
        streams += [io.TextIOWrapper(io.BytesIO(), encoding="ascii") for _ in range(2)]
        with unittest.mock.patch.multiple(sys, stdin=streams[0], stdout=streams[1], stderr=streams[2]):
            status = plurals.main(["english-plurals.py", *words])
        said = []
        for stream in streams[1:]:
            stream.flush()
            said.append(stream.buffer.getvalue().decode("utf-8"))
        return status, said[0], said[1]

    def test_fill_writes_the_catalogue_and_names_each_new_pair(self):
        text = HEAD + plain("a", "Übersicht") + counted("b", "%n places") + TAIL
        status, out, errors = self.run_with(text, "fill")
        self.assertEqual(status, 0)
        self.assertEqual(out, HEAD + plain("a", "Übersicht") + counted("b", "%n places", ["%n place", "%n places"]) + TAIL)
        self.assertEqual(errors, "english-plurals: b: %n place | %n places\n")

    def test_a_refused_text_fails_the_run_but_the_catalogue_still_comes_out(self):
        text = HEAD + counted("b", "%n place(s)") + TAIL
        status, out, errors = self.run_with(text, "fill")
        self.assertEqual((status, out), (1, text))
        self.assertEqual(errors, "english-plurals: b: write a plain plural, not (s)\n")

    def test_what_is_no_catalogue_fails_and_writes_nothing(self):
        status, out, errors = self.run_with("<xml/>", "fill")
        self.assertEqual((status, out, errors), (1, "", "english-plurals: not a TS catalogue\n"))

    def test_wrong_use_prints_the_usage(self):
        for words in ((), ("fill", "more"), ("check",)):
            status, out, errors = self.run_with(HEAD + TAIL, *words)
            self.assertEqual((status, out), (2, ""), msg=words)
            self.assertIn("usage: english-plurals.py fill", errors)


if __name__ == "__main__":
    unittest.main()
