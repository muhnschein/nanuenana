// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import "Words.js" as Words

Item {
    property var view: null

    width: parent ? parent.width : 0
    height: box.height + Theme.paddingLarge
    visible: Words.replaying(view)

    NoteBox {
        id: box

        alarming: true
        title: Words.testData()
        //% "These are not real warnings."
        text: qsTrId("nanunana-test-data-text")
    }
}
