// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "Words.js" as Words

BackgroundItem {
    id: item

    property var warning
    property bool ended
    property bool dim
    property double nowMs
    readonly property bool cancelled: ended || (!!warning && warning.kind === "cancel")

    function timeText() {
        if (!warning)
            return ""
        if (cancelled)
            return Words.when(warning.sentMs, nowMs)
        if (warning.expiresMs > 0) {
            //% "until %1"
            return qsTrId("nanunana-warning-until").arg(Words.when(warning.expiresMs, nowMs))
        }
        //% "since %1"
        return qsTrId("nanunana-warning-since").arg(Words.when(warning.sentMs, nowMs))
    }

    function subtitle() {
        if (!warning)
            return ""
        return warning.sender.length > 0 ? warning.sender + " · " + timeText() : timeText()
    }

    width: parent ? parent.width : 0
    height: Math.max(Theme.itemSizeLarge, body.height + 2 * Theme.paddingMedium)
    opacity: dim || ended ? 0.6 : 1.0

    CategoryIcon {
        id: icon

        x: Theme.horizontalPageMargin
        y: Theme.paddingMedium
        icon: item.warning ? item.warning.icon : ""
        level: item.warning ? item.warning.level : 0
        ended: item.cancelled
    }

    Column {
        id: body

        anchors {
            left: icon.right
            leftMargin: Theme.paddingMedium
            right: parent.right
            rightMargin: Theme.horizontalPageMargin
        }
        y: Theme.paddingMedium

        Label {
            objectName: "warningHeadline"
            width: parent.width
            text: item.warning ? item.warning.headline : ""
            color: item.highlighted ? Theme.highlightColor : Theme.primaryColor
            wrapMode: Text.Wrap
            maximumLineCount: 3
            truncationMode: TruncationMode.Elide
        }

        Label {
            objectName: "warningSubtitle"
            width: parent.width
            text: item.subtitle()
            color: item.highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor
            font.pixelSize: Theme.fontSizeSmall
            truncationMode: TruncationMode.Fade
        }

        LevelLine {
            width: parent.width
            visible: !item.cancelled
            level: item.warning ? item.warning.level : 0
            detail: item.warning ? Words.categoryName(item.warning.category) : ""
        }

        Label {
            width: parent.width
            visible: item.cancelled
            //% "All-clear"
            text: qsTrId("nanunana-all-clear") + (item.warning ? " · " + Words.categoryName(item.warning.category) : "")
            color: Theme.secondaryColor
            font.pixelSize: Theme.fontSizeSmall
            truncationMode: TruncationMode.Fade
        }
    }
}
