// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "Words.js" as Words

Row {
    id: line

    property int level
    property string detail

    spacing: Theme.paddingSmall

    LevelBars {
        id: bars

        anchors.verticalCenter: parent.verticalCenter
        level: line.level
    }

    Label {
        anchors.verticalCenter: parent.verticalCenter
        visible: line.level > 0
        text: Words.levelName(line.level)
        color: bars.lit.length > 0 ? bars.lit : Theme.secondaryColor
        font.pixelSize: Theme.fontSizeSmall
    }

    Label {
        anchors.verticalCenter: parent.verticalCenter
        width: Math.min(implicitWidth, Math.max(0, line.width - x))
        visible: line.detail.length > 0
        text: line.level > 0 ? "· " + line.detail : line.detail
        color: Theme.secondaryColor
        font.pixelSize: Theme.fontSizeSmall
        truncationMode: TruncationMode.Fade
    }
}
