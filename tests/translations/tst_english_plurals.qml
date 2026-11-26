// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6

QtObject {
    property bool failed: false
    property bool reachedEnd

    function same(got, want, what) {
        if (got === want || failed)
            return
        failed = true
        console.error("FAILED: " + what + ": \"" + got + "\", not \"" + want + "\"")
    }

    function plural(id, one, other) {
        same(qsTrId(id, 1), one, id + " for 1")
        same(qsTrId(id, 2), other.replace("%n", "2"), id + " for 2")
        same(qsTrId(id, 0), other.replace("%n", "0"), id + " for 0")
        same(qsTrId(id, 11), other.replace("%n", "11"), id + " for 11")
    }

    Component.onCompleted: {
        same(qsTrId("nanunana-app-name"), "Nanünana", "the catalogue is installed")
        same(qsTrId("nanunana-level-danger"), "Danger", "a plain string")
        same(qsTrId("nanunana-about-licence-text").split("\n").length, 2, "line breaks survive")

        plural("nanunana-places-count", "1 warning", "%n warnings")
        plural("nanunana-cover-places", "1 place", "%n places")
        plural("nanunana-cover-more", "+ 1 more", "+ %n more")
        plural("nanunana-settings-minutes", "1 minute", "%n minutes")
        reachedEnd = true
    }
}
