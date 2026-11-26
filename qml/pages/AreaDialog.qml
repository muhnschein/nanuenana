// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"

Dialog {
    id: dialog

    property int placeId
    property string name
    property string district
    property string districtName
    property string stateName
    property int km: 5
    readonly property bool around: mode.currentIndex === 0
    readonly property int chosenKm: around ? Math.round(distance.value) : 0

    allowedOrientations: Orientation.All
    onAccepted: {
        if (placeId > 0) {
            App.setArea(placeId, chosenKm)
        } else if (App.addPlace(name, district, chosenKm) === "") {
            App.check()
        }
    }

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height + Theme.paddingLarge

        Column {
            id: column

            width: parent.width

            DialogHeader {
                //% "Save"
                acceptText: qsTrId("nanunana-area-save")
            }

            Label {
                objectName: "areaName"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                text: dialog.name
                color: Theme.highlightColor
                font.pixelSize: Theme.fontSizeLarge
                wrapMode: Text.Wrap
            }

            Label {
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                height: implicitHeight + Theme.paddingMedium
                text: dialog.districtName + ", " + dialog.stateName
                color: Theme.secondaryHighlightColor
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.Wrap
            }

            ComboBox {
                id: mode

                objectName: "areaMode"
                //% "Warn for"
                label: qsTrId("nanunana-area-warn-for")
                currentIndex: dialog.km === 0 ? 1 : 0

                menu: ContextMenu {
                    MenuItem {
                        //% "Around this place"
                        text: qsTrId("nanunana-area-around")
                    }

                    MenuItem {
                        //% "Whole district (%1)"
                        text: qsTrId("nanunana-area-whole").arg(dialog.districtName)
                    }
                }
            }

            Slider {
                id: distance

                objectName: "areaDistance"
                width: parent.width
                visible: dialog.around
                minimumValue: 1
                maximumValue: 25
                stepSize: 1
                value: dialog.km > 0 ? dialog.km : 5
                //% "%1 km"
                valueText: qsTrId("nanunana-area-km").arg(Math.round(value))
                //% "Distance, 1 to 25 km"
                label: qsTrId("nanunana-area-distance")
            }

            Label {
                objectName: "areaExplanation"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                height: implicitHeight + Theme.paddingLarge
                text: dialog.around
                    //% "You get every warning whose area comes within %1 km of the centre of %2. Warnings for the whole district or the whole Land always count."
                    ? qsTrId("nanunana-area-explain-around").arg(dialog.chosenKm).arg(dialog.name)
                    //% "You get every warning for %1."
                    : qsTrId("nanunana-area-explain-whole").arg(dialog.districtName)
                color: Theme.secondaryColor
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.Wrap
            }

            NoteBox {
                visible: dialog.around
                //% "A larger distance means more warnings that do not concern you. A smaller one can miss a warning drawn tightly around a neighbouring street."
                text: qsTrId("nanunana-area-tradeoff")
            }
        }

        VerticalScrollDecorator {}
    }
}
