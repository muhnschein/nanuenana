// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0

ComboBox {
    id: box

    property var values: []
    property var labels: []
    property var value

    signal chosen(var value)

    currentIndex: values.indexOf(value)

    menu: ContextMenu {
        Repeater {
            model: box.labels

            MenuItem {
                text: modelData
                onClicked: box.chosen(box.values[index])
            }
        }
    }
}
