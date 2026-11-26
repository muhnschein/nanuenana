// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "Words.js" as Words

HighlightImage {
    property string category
    property int level
    property bool ended
    readonly property string lit: Words.levelColor(level, Theme.colorScheme === Theme.LightOnDark)
    readonly property var names: ({ "civil": "cat-civil", "police": "cat-police", "weather": "cat-weather", "flood": "cat-flood" })

    source: Qt.resolvedUrl("../icons/" + (ended ? "all-clear" : (names[category] || "cat-civil")) + ".svg")
    sourceSize.width: Theme.iconSizeMedium
    sourceSize.height: Theme.iconSizeMedium
    width: Theme.iconSizeMedium
    height: Theme.iconSizeMedium
    color: !ended && lit.length > 0 ? lit : Theme.primaryColor
    highlightColor: color
}
