#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
import re
import sys

MESSAGE = re.compile(r'<message id="(?P<id>[^"]*)"(?P<numerus> numerus="yes")?>(?P<body>.*?)</message>', re.S)
TRANSLATION = re.compile(r"<translation(?P<attrs>[^>]*)>(?P<text>.*?)</translation>", re.S)
FORM = re.compile(r"<numerusform>(.*?)</numerusform>", re.S)
SOURCE = re.compile(r"<source>(.*?)</source>", re.S)
PLACEHOLDER = re.compile(r"%(?:n|L?\d+)")


def gap(message):
    name = message.group("id")
    translation = TRANSLATION.search(message.group("body"))
    if translation is None or "unfinished" in translation.group("attrs"):
        return f"{name}: not translated"
    texts = FORM.findall(translation.group("text")) if message.group("numerus") else [translation.group("text")]
    if not texts or not all(t.strip() for t in texts):
        return f"{name}: empty translation"
    source = SOURCE.search(message.group("body"))
    wanted = sorted(set(PLACEHOLDER.findall(source.group(1)))) if source else []
    if any(sorted(set(PLACEHOLDER.findall(one))) != wanted for one in texts):
        return f"{name}: placeholders differ from the source ({', '.join(wanted) or 'none'})"
    return None


def gaps(text):
    return [found for found in map(gap, MESSAGE.finditer(text)) if found]


def main(catalogue, errors):
    found = gaps(catalogue.read())
    for one in found:
        print(f"untranslated: {one}", file=errors)
    return 1 if found else 0


if __name__ == "__main__":
    sys.stdin.reconfigure(encoding="utf-8")
    sys.exit(main(sys.stdin, sys.stderr))
