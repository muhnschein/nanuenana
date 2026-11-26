// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"
import "../components/Words.js" as Words

CoverBackground {
    id: cover

    readonly property var view: JSON.parse(App.stateJson)
    readonly property var summary: Words.summary(view)
    readonly property double nowMs: { App.stateJson; return Date.now() }
    readonly property string mode: {
        if (summary.places === 0)
            return "empty"
        if (summary.stale !== null)
            return "stale"
        return summary.level > 0 ? "warned" : "clear"
    }

    function title() {
        switch (mode) {
        case "stale":
            //% "Not updated"
            return qsTrId("nanunana-cover-stale")
        case "warned":
            return Words.levelName(summary.level)
        case "clear":
            //% "No warnings"
            return qsTrId("nanunana-no-warnings")
        default:
            //% "Nanünana"
            return qsTrId("nanunana-app-name")
        }
    }

    function first() {
        switch (mode) {
        case "stale":
            if (summary.stale.updatedMs <= 0)
                return ""
            //% "since %1"
            return qsTrId("nanunana-warning-since").arg(Words.when(summary.stale.updatedMs, nowMs))
        case "warned":
            return summary.top.name
        case "clear":
            //% "%n places"
            return qsTrId("nanunana-cover-places", summary.places)
        default:
            //% "No places yet"
            return qsTrId("nanunana-places-empty")
        }
    }

    function second() {
        switch (mode) {
        case "stale":
            //% "Warnings may be missing"
            return qsTrId("nanunana-warnings-missing")
        case "warned":
            return summary.top.headline
        case "clear":
            //% "Checked %1"
            return qsTrId("nanunana-places-checked").arg(Words.when(summary.checkedMs, nowMs))
        default:
            return ""
        }
    }

    function third() {
        if (mode !== "warned" || summary.warnings <= 1)
            return ""
        //% "+ %n more"
        return qsTrId("nanunana-cover-more", summary.warnings - 1)
    }

    Column {
        anchors {
            top: parent.top
            topMargin: Theme.paddingLarge
            left: parent.left
            leftMargin: Theme.paddingLarge
            right: parent.right
            rightMargin: Theme.paddingLarge
        }
        spacing: Theme.paddingSmall

        LevelBars {
            id: bars

            big: true
            visible: cover.mode === "warned" || cover.mode === "clear"
            level: cover.mode === "warned" ? cover.summary.level : 0
        }

        HighlightImage {
            visible: cover.mode === "stale"
            source: "image://theme/icon-m-warning"
            color: Theme.errorColor
        }

        Label {
            objectName: "coverTitle"
            width: parent.width
            text: cover.title()
            color: cover.mode === "stale" ? Theme.errorColor : (cover.mode === "warned" && bars.lit.length > 0 ? bars.lit : Theme.highlightColor)
            font.pixelSize: Theme.fontSizeLarge
            truncationMode: TruncationMode.Fade
        }

        Label {
            objectName: "coverFirst"
            width: parent.width
            text: cover.first()
            truncationMode: TruncationMode.Fade
        }

        Label {
            objectName: "coverSecond"
            width: parent.width
            text: cover.second()
            color: Theme.secondaryColor
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.Wrap
            maximumLineCount: 3
            truncationMode: TruncationMode.Elide
        }

        Label {
            width: parent.width
            visible: text.length > 0
            text: cover.third()
            color: Theme.secondaryColor
            font.pixelSize: Theme.fontSizeExtraSmall
            truncationMode: TruncationMode.Fade
        }
    }

    CoverActionList {
        enabled: cover.mode !== "empty"

        CoverAction {
            iconSource: "image://theme/icon-cover-refresh"
            onTriggered: App.requestCheck()
        }
    }
}
