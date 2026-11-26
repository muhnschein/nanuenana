// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

Item {
    property alias text: label.text

    width: parent ? parent.width : 0
    height: label.height + Theme.paddingMedium + Theme.paddingLarge

    Label {
        id: label

        x: Theme.horizontalPageMargin
        y: Theme.paddingMedium
        width: parent.width - 2 * x
        color: Theme.secondaryColor
        font.pixelSize: Theme.fontSizeExtraSmall
        wrapMode: Text.Wrap
    }
}
