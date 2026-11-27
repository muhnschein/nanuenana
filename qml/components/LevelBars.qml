// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "Words.js" as Words

Row {
    id: bars

    property int level
    property bool big
    readonly property real unit: (big ? 1.75 : 1) * Theme.pixelRatio
    readonly property string lit: Words.levelColor(level, Theme.colorScheme === Theme.LightOnDark)

    spacing: Math.round(2 * unit)
    height: Math.round(18 * unit)

    Repeater {
        model: 4

        Rectangle {
            anchors.bottom: parent.bottom
            width: Math.round(5 * bars.unit)
            height: Math.round((6 + 4 * index) * bars.unit)
            color: index < bars.level && bars.lit.length > 0 ? bars.lit : Theme.rgba(Theme.primaryColor, 0.28)
        }
    }
}
