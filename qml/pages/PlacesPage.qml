// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"
import "../components/Words.js" as Words

Page {
    id: page

    readonly property var view: JSON.parse(App.stateJson)
    readonly property var places: view !== null ? view.places : []
    readonly property double nowMs: { App.stateJson; return Date.now() }
    property bool noticeShown

    function headline(place) {
        if (place.current && place.lostSinceMs > 0) {
            //% "No position since %1"
            return qsTrId("nanunana-places-lost").arg(Words.when(place.lostSinceMs, nowMs))
        }
        if (place.stale) {
            if (place.updatedMs > 0) {
                //% "Not updated since %1"
                return qsTrId("nanunana-places-stale").arg(Words.when(place.updatedMs, nowMs))
            }
            //% "Not updated yet"
            return qsTrId("nanunana-places-never")
        }
        if (place.warnings.length === 0) {
            //% "No warnings"
            return qsTrId("nanunana-no-warnings")
        }
        return place.headline
    }

    function detail(place) {
        if (place.stale) {
            //% "Warnings may be missing"
            return qsTrId("nanunana-warnings-missing")
        }
        if (place.warnings.length === 0) {
            //% "Checked %1"
            return qsTrId("nanunana-places-checked").arg(Words.when(place.updatedMs, nowMs))
        }
        //% "%n warnings"
        return qsTrId("nanunana-places-count", place.warnings.length)
    }

    function locationText() {
        if (view === null)
            return ""
        if (view.location === "waiting") {
            //% "Current location: waiting for a position"
            return qsTrId("nanunana-places-location-waiting")
        }
        if (view.location === "outside") {
            //% "Current location: outside Germany. Nanünana shows warnings for Germany only."
            return qsTrId("nanunana-places-location-outside")
        }
        return ""
    }

    allowedOrientations: Orientation.All
    onStatusChanged: {
        if (status !== PageStatus.Active || noticeShown || view === null || view.noticeAccepted)
            return
        noticeShown = true
        notice.start()
    }

    Timer {
        id: notice

        interval: 0
        onTriggered: pageStack.push(Qt.resolvedUrl("NoticeDialog.qml"), {}, PageStackAction.Immediate)
    }

    SilicaListView {
        id: list

        objectName: "placeList"
        anchors.fill: parent
        model: page.places.length

        PullDownMenu {
            busy: App.checking

            MenuItem {
                //% "Settings"
                text: qsTrId("nanunana-settings-title")
                onClicked: pageStack.push(Qt.resolvedUrl("SettingsPage.qml"))
            }

            MenuItem {
                objectName: "updateNow"
                //% "Update now"
                text: qsTrId("nanunana-update-now")
                enabled: !App.checking
                onClicked: App.requestCheck()
            }

            MenuItem {
                objectName: "addPlace"
                //% "Add place"
                text: qsTrId("nanunana-add-place")
                onClicked: pageStack.push(Qt.resolvedUrl("AddPlacePage.qml"))
            }
        }

        header: Column {
            width: list.width

            PageHeader {
                objectName: "placesHeader"
                //% "Places"
                title: qsTrId("nanunana-places-title")
                description: {
                    if (page.view === null || page.view.checkedMs <= 0 || page.places.length === 0)
                        return ""
                    //% "Updated %1"
                    return qsTrId("nanunana-places-updated").arg(Words.when(page.view.checkedMs, page.nowMs))
                }
            }

            Label {
                objectName: "locationNote"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                height: visible ? implicitHeight + Theme.paddingLarge : 0
                visible: text.length > 0
                text: page.locationText()
                color: Theme.secondaryHighlightColor
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.Wrap
            }
        }

        delegate: ListItem {
            id: row

            readonly property var place: page.places[index] || null
            readonly property bool alarming: place !== null && (place.stale || (place.current && place.lostSinceMs > 0))

            objectName: "placeRow"
            width: list.width
            contentHeight: Math.max(Theme.itemSizeLarge, body.height + 2 * Theme.paddingMedium)
            menu: place !== null && !place.current ? placeMenu : null
            onClicked: pageStack.push(Qt.resolvedUrl("PlacePage.qml"), { "placeId": place.id })

            Column {
                id: body

                anchors {
                    left: parent.left
                    leftMargin: Theme.horizontalPageMargin
                    right: area.left
                    rightMargin: Theme.paddingMedium
                }
                y: Theme.paddingMedium

                Row {
                    width: parent.width
                    spacing: Theme.paddingSmall

                    HighlightImage {
                        id: pin

                        anchors.verticalCenter: parent.verticalCenter
                        visible: row.place !== null && row.place.current
                        source: "image://theme/icon-m-whereami"
                        sourceSize.width: Theme.iconSizeSmall
                        sourceSize.height: Theme.iconSizeSmall
                        width: visible ? Theme.iconSizeSmall : 0
                        height: Theme.iconSizeSmall
                        color: Theme.highlightColor
                    }

                    Label {
                        objectName: "placeName"
                        width: parent.width - (pin.visible ? pin.width + parent.spacing : 0)
                        text: row.place !== null ? row.place.name : ""
                        color: row.highlighted ? Theme.highlightColor : Theme.primaryColor
                        truncationMode: TruncationMode.Fade
                    }
                }

                Label {
                    objectName: "placeHeadline"
                    width: parent.width
                    text: row.place !== null ? page.headline(row.place) : ""
                    color: row.alarming ? Theme.errorColor : (row.highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor)
                    font.pixelSize: Theme.fontSizeSmall
                    truncationMode: TruncationMode.Fade
                }

                Row {
                    width: parent.width
                    spacing: Theme.paddingSmall

                    HighlightImage {
                        id: caution

                        anchors.verticalCenter: parent.verticalCenter
                        visible: row.place !== null && row.place.stale
                        source: "image://theme/icon-s-warning"
                        width: visible ? implicitWidth : 0
                        color: Theme.errorColor
                    }

                    LevelLine {
                        objectName: "placeLevel"
                        width: parent.width - (caution.visible ? caution.width + parent.spacing : 0)
                        level: row.place !== null && !row.place.stale ? row.place.level : 0
                        detail: row.place !== null ? page.detail(row.place) : ""
                    }
                }
            }

            Label {
                id: area

                objectName: "placeArea"
                anchors {
                    right: parent.right
                    rightMargin: Theme.horizontalPageMargin
                }
                y: Theme.paddingMedium
                width: Math.min(implicitWidth, row.width / 4)
                text: Words.areaText(row.place)
                color: Theme.secondaryColor
                font.pixelSize: Theme.fontSizeSmall
                horizontalAlignment: Text.AlignRight
                wrapMode: Text.Wrap
            }

            Component {
                id: placeMenu

                ContextMenu {
                    MenuItem {
                        //% "Change area"
                        text: qsTrId("nanunana-place-change-area")
                        onClicked: pageStack.push(Qt.resolvedUrl("AreaDialog.qml"), {
                            "placeId": row.place.id,
                            "name": row.place.name,
                            "districtName": row.place.districtName,
                            "stateName": row.place.stateName,
                            "km": row.place.km
                        })
                    }

                    MenuItem {
                        //% "Remove"
                        text: qsTrId("nanunana-place-remove")
                        onClicked: {
                            var id = row.place.id
                            //% "Removing"
                            row.remorseAction(qsTrId("nanunana-place-removing"), function() { App.removePlace(id) })
                        }
                    }
                }
            }
        }

        footer: Unofficial {
            width: list.width
            visible: page.places.length > 0
            //% "Unofficial app. In an emergency follow sirens, cell broadcast, radio and the authorities."
            text: qsTrId("nanunana-unofficial-short")
        }

        ViewPlaceholder {
            objectName: "placesEmpty"
            enabled: page.places.length === 0
            //% "No places yet"
            text: qsTrId("nanunana-places-empty")
            //% "Pull down to add a place. Warnings at your current location can be turned on in Settings. Without either you get no alerts."
            hintText: qsTrId("nanunana-places-empty-hint")
        }

        VerticalScrollDecorator {}
    }
}
