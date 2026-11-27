// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0

Dialog {
    id: dialog

    allowedOrientations: Orientation.All
    onAccepted: App.acceptNotice()
    onRejected: Qt.quit()

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height + Theme.paddingLarge

        Column {
            id: column

            width: parent.width

            DialogHeader {
                //% "I understand"
                acceptText: qsTrId("nanunana-notice-accept")
                //% "Quit"
                cancelText: qsTrId("nanunana-notice-quit")
            }

            HighlightImage {
                anchors.right: parent.right
                anchors.rightMargin: Theme.horizontalPageMargin
                source: "image://theme/icon-l-attention"
                color: Theme.highlightColor
            }

            Label {
                objectName: "noticeTitle"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                //% "Nanünana is not an official app"
                text: qsTrId("nanunana-notice-title")
                color: Theme.highlightColor
                font.pixelSize: Theme.fontSizeExtraLarge
                font.family: Theme.fontFamilyHeading
                horizontalAlignment: Text.AlignRight
                wrapMode: Text.Wrap
            }

            Item {
                width: 1
                height: Theme.paddingLarge
            }

            Repeater {
                model: [
                    //% "It shows warnings that authorities publish. Volunteers make it. No authority runs, checks or endorses it."
                    qsTrId("nanunana-notice-what"),
                    //% "It can be late, and it can miss a warning. It asks for new warnings every few minutes, only while it is open or minimised, and only with a data connection."
                    qsTrId("nanunana-notice-limits"),
                    //% "In an emergency follow the official channels first: sirens, cell broadcast on your phone, radio and television, and what the authorities on site tell you."
                    qsTrId("nanunana-notice-official")
                ]

                Label {
                    x: Theme.horizontalPageMargin
                    width: column.width - 2 * x
                    height: implicitHeight + Theme.paddingLarge
                    text: modelData
                    wrapMode: Text.Wrap
                }
            }
        }

        VerticalScrollDecorator {}
    }
}
