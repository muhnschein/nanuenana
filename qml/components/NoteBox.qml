// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

Item {
    id: box

    property string title
    property string text
    property bool alarming

    x: Theme.horizontalPageMargin
    width: parent ? parent.width - 2 * x : 0
    height: content.height + 2 * Theme.paddingMedium

    Rectangle {
        anchors.fill: parent
        color: Theme.rgba(Theme.highlightBackgroundColor, 0.13)
    }

    HighlightImage {
        id: icon

        x: Theme.paddingMedium
        y: Theme.paddingMedium
        source: "image://theme/icon-s-warning"
        color: box.alarming ? Theme.errorColor : Theme.highlightColor
    }

    Column {
        id: content

        anchors {
            left: icon.right
            leftMargin: Theme.paddingMedium
            right: parent.right
            rightMargin: Theme.paddingMedium
        }
        y: Theme.paddingMedium

        Label {
            objectName: "noteTitle"
            width: parent.width
            visible: box.title.length > 0
            text: box.title
            color: Theme.errorColor
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.Wrap
        }

        Label {
            objectName: "noteText"
            width: parent.width
            text: box.text
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.Wrap
        }
    }
}
