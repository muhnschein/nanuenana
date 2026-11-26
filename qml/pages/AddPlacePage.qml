// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"

Page {
    id: page

    property string query
    readonly property var hits: { App.stateJson; return JSON.parse(App.search(query)) }

    allowedOrientations: Orientation.All

    SilicaListView {
        id: list

        anchors.fill: parent
        currentIndex: -1
        model: page.hits.length

        header: Column {
            width: list.width

            PageHeader {
                //% "Add place"
                title: qsTrId("nanunana-add-place")
            }

            SearchField {
                id: field

                objectName: "placeSearch"
                width: parent.width
                //% "Town, village or postcode"
                placeholderText: qsTrId("nanunana-add-search")
                inputMethodHints: Qt.ImhNoPredictiveText
                EnterKey.iconSource: "image://theme/icon-m-enter-close"
                EnterKey.onClicked: focus = false
                onTextChanged: page.query = text
                Component.onCompleted: forceActiveFocus()
            }
        }

        delegate: BackgroundItem {
            id: row

            readonly property var hit: page.hits[index] || null

            objectName: "hitRow"
            width: list.width
            height: Theme.itemSizeMedium
            enabled: hit !== null && !hit.saved
            opacity: enabled ? 1.0 : Theme.opacityLow
            onClicked: pageStack.push(Qt.resolvedUrl("AreaDialog.qml"), {
                "name": hit.name,
                "district": hit.district,
                "districtName": hit.districtName,
                "stateName": hit.stateName,
                "acceptDestination": pageStack.previousPage(page),
                "acceptDestinationAction": PageStackAction.Pop
            })

            Column {
                anchors {
                    left: parent.left
                    leftMargin: Theme.horizontalPageMargin
                    right: code.left
                    rightMargin: Theme.paddingMedium
                    verticalCenter: parent.verticalCenter
                }

                Label {
                    objectName: "hitName"
                    width: parent.width
                    text: row.hit !== null ? Theme.highlightText(row.hit.name, page.query, Theme.highlightColor) : ""
                    textFormat: Text.StyledText
                    color: row.highlighted ? Theme.highlightColor : Theme.primaryColor
                    truncationMode: TruncationMode.Fade
                }

                Label {
                    objectName: "hitDistrict"
                    width: parent.width
                    text: row.hit !== null ? row.hit.districtName + ", " + row.hit.stateName : ""
                    color: row.highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor
                    font.pixelSize: Theme.fontSizeSmall
                    truncationMode: TruncationMode.Fade
                }
            }

            Label {
                id: code

                anchors {
                    right: parent.right
                    rightMargin: Theme.horizontalPageMargin
                    verticalCenter: parent.verticalCenter
                }
                //% "Saved"
                text: row.hit === null ? "" : (row.hit.saved ? qsTrId("nanunana-add-saved") : row.hit.postcode)
                color: Theme.secondaryColor
                font.pixelSize: Theme.fontSizeSmall
            }
        }

        footer: Unofficial {
            width: list.width
            //% "The place list is on your phone: what you type is not sent anywhere. Germany only for now."
            text: qsTrId("nanunana-add-private")
        }

        ViewPlaceholder {
            enabled: page.query.length >= 2 && page.hits.length === 0
            //% "No place found"
            text: qsTrId("nanunana-add-none")
            //% "Try the postcode, or a larger town nearby."
            hintText: qsTrId("nanunana-add-none-hint")
        }

        VerticalScrollDecorator {}
    }
}
