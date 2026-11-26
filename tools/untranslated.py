#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import re
import sys

MESSAGE = re.compile(r'<message id="(?P<id>[^"]*)"(?P<numerus> numerus="yes")?>(?P<body>.*?)</message>', re.S)
TRANSLATION = re.compile(r"<translation(?P<attrs>[^>]*)>(?P<text>.*?)</translation>", re.S)
FORM = re.compile(r"<numerusform>(.*?)</numerusform>", re.S)
SOURCE = re.compile(r"<source>(.*?)</source>", re.S)
PLACEHOLDER = re.compile(r"%(?:n|L?\d+)")


def gaps(text):
    found = []
    for message in MESSAGE.finditer(text):
        name = message.group("id")
        translation = TRANSLATION.search(message.group("body"))
        if translation is None or "unfinished" in translation.group("attrs"):
            found.append(f"{name}: not translated")
            continue
        texts = FORM.findall(translation.group("text")) if message.group("numerus") else [translation.group("text")]
        if not texts or not all(t.strip() for t in texts):
            found.append(f"{name}: empty translation")
            continue
        source = SOURCE.search(message.group("body"))
        wanted = sorted(set(PLACEHOLDER.findall(source.group(1)))) if source else []
        for one in texts:
            if sorted(set(PLACEHOLDER.findall(one))) != wanted:
                found.append(f"{name}: placeholders differ from the source ({', '.join(wanted) or 'none'})")
                break
    return found


def main(argv):
    if len(argv) != 2:
        print("usage: untranslated.py CATALOGUE.ts", file=sys.stderr)
        return 2
    with open(argv[1], encoding="utf-8") as catalogue:
        found = gaps(catalogue.read())
    for gap in found:
        print(f"untranslated: {gap}", file=sys.stderr)
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
