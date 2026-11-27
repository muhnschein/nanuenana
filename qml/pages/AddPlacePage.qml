// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"
import "../components/Words.js" as Words

Page {
    id: page

    property string query

    function refresh() {
        var found = JSON.parse(App.search(query))
        hits.clear()
        for (var i = 0; i < found.length; ++i)
            hits.append(found[i])
    }

    allowedOrientations: Orientation.All
    onQueryChanged: refresh()

    ListModel {
        id: hits
    }

    Connections {
        target: App
        onStateChanged: page.refresh()
    }

    SilicaListView {
        id: list

        objectName: "hitList"
        anchors.fill: parent
        currentIndex: -1
        model: hits

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

            objectName: "hitRow"
            width: list.width
            height: Theme.itemSizeMedium
            enabled: !model.saved
            opacity: enabled ? 1.0 : Theme.opacityLow
            onClicked: pageStack.push(Qt.resolvedUrl("AreaDialog.qml"), {
                "name": model.name,
                "district": model.district,
                "districtName": model.districtName,
                "stateName": model.stateName,
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
                    text: Words.marked(model.name, model.matchStart, model.matchLength, Theme.highlightColor)
                    textFormat: Text.StyledText
                    color: row.highlighted ? Theme.highlightColor : Theme.primaryColor
                    truncationMode: TruncationMode.Fade
                }

                Label {
                    objectName: "hitDistrict"
                    width: parent.width
                    text: model.districtName + ", " + model.stateName
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
                text: model.saved ? qsTrId("nanunana-add-saved") : model.postcode
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
            enabled: page.query.length >= 2 && hits.count === 0
            //% "No place found"
            text: qsTrId("nanunana-add-none")
            //% "Try the postcode, or a larger town nearby."
            hintText: qsTrId("nanunana-add-none-hint")
        }

        VerticalScrollDecorator {}
    }
}
