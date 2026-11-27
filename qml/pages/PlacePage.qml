// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"
import "../components/Words.js" as Words

Page {
    id: page

    property int placeId
    readonly property var view: JSON.parse(App.stateJson)
    readonly property var settings: JSON.parse(App.settingsJson)
    readonly property var place: Words.placeOf(view, placeId)
    readonly property double nowMs: { App.stateJson; return Date.now() }
    readonly property bool lost: place !== null && place.current && place.lostSinceMs > 0
    readonly property bool stale: place !== null && place.stale
    readonly property bool calm: place !== null && !stale && !lost && place.warnings.length === 0 && place.ended.length === 0

    function active(i) {
        return place !== null && place.warnings[i] ? place.warnings[i] : null
    }

    function past(i) {
        return place !== null && place.ended[i] ? place.ended[i] : null
    }

    function nextCheck() {
        var minutes = settings !== null ? settings.interval : 5
        return Words.clock(place.updatedMs + minutes * 60000)
    }

    function description() {
        if (place === null)
            return ""
        var area = Words.areaText(place)
        if (lost) {
            //% "%1 · no position since %2"
            return qsTrId("nanunana-place-lost").arg(area).arg(Words.when(place.lostSinceMs, nowMs))
        }
        if (stale && place.updatedMs > 0) {
            //% "%1 · not updated since %2"
            return qsTrId("nanunana-place-stale").arg(area).arg(Words.when(place.updatedMs, nowMs))
        }
        if (place.updatedMs <= 0)
            return area
        //% "%1 · updated %2"
        return qsTrId("nanunana-place-updated").arg(area).arg(Words.when(place.updatedMs, nowMs))
    }

    function footerText() {
        if (place === null)
            return ""
        if (stale || lost) {
            //% "Unofficial display of official warnings. Follow sirens, cell broadcast and radio."
            return qsTrId("nanunana-place-foot-stale")
        }
        if (place.warnings.length === 0) {
            //% "Unofficial display of official warnings. No warning here does not mean no danger: follow sirens, cell broadcast and radio."
            return qsTrId("nanunana-place-foot-clear")
        }
        //% "Unofficial display of official warnings. Last successful update %1, next at about %2."
        return qsTrId("nanunana-place-foot").arg(Words.when(place.updatedMs, nowMs)).arg(nextCheck())
    }

    allowedOrientations: Orientation.All

    SilicaFlickable {
        id: flick

        anchors.fill: parent
        contentHeight: Math.max(column.height, height)

        PullDownMenu {
            busy: App.checking

            MenuItem {
                //% "Update now"
                text: qsTrId("nanunana-update-now")
                enabled: !App.checking
                onClicked: App.requestCheck()
            }
        }

        Column {
            id: column

            width: parent.width

            PageHeader {
                objectName: "placeHeader"
                title: page.place !== null ? page.place.name : ""
                description: page.description()
            }

            TestData {
                objectName: "testData"
                view: page.view
            }

            NoteBox {
                objectName: "lostBox"
                visible: page.lost
                alarming: true
                //% "You may be somewhere else by now"
                title: qsTrId("nanunana-place-lost-title")
                //% "Nanünana has had no position since %1. The warnings below are for %2, where you were then. Check that positioning is on in the phone settings."
                text: page.lost ? qsTrId("nanunana-place-lost-text").arg(Words.when(page.place.lostSinceMs, page.nowMs)).arg(page.place.name) : ""
            }

            NoteBox {
                objectName: "staleBox"
                visible: page.stale && !page.lost
                alarming: true
                //% "Warnings may be missing"
                title: qsTrId("nanunana-warnings-missing")
                text: {
                    if (!page.stale)
                        return ""
                    if (page.place.updatedMs <= 0) {
                        //% "Nanünana has not reached warnung.bund.de yet. It tries again at the next check."
                        return qsTrId("nanunana-place-never-text")
                    }
                    //% "Nanünana could not reach warnung.bund.de since %1. It tries again at the next check."
                    return qsTrId("nanunana-place-stale-text").arg(Words.when(page.place.updatedMs, page.nowMs))
                }
            }

            Item {
                width: parent.width
                height: retry.height + 2 * Theme.paddingLarge
                visible: page.stale || page.lost

                Button {
                    id: retry

                    objectName: "tryNow"
                    anchors.centerIn: parent
                    //% "Try now"
                    text: qsTrId("nanunana-place-try")
                    enabled: !App.checking
                    onClicked: App.requestCheck()
                }
            }

            SectionHeader {
                visible: page.place !== null && (page.stale || page.lost) && page.place.warnings.length > 0 && page.place.updatedMs > 0
                text: {
                    if (!visible)
                        return ""
                    if (page.lost) {
                        //% "%1, checked %2"
                        return qsTrId("nanunana-place-lost-section").arg(page.place.name).arg(Words.when(page.place.updatedMs, page.nowMs))
                    }
                    //% "As of %1"
                    return qsTrId("nanunana-place-as-of").arg(Words.when(page.place.updatedMs, page.nowMs))
                }
            }

            Repeater {
                model: page.place !== null ? page.place.warnings.length : 0

                WarningItem {
                    objectName: "warningRow"
                    warning: page.active(index)
                    dim: page.stale
                    nowMs: page.nowMs
                    onClicked: pageStack.push(Qt.resolvedUrl("WarningPage.qml"), { "warningId": warning.id })
                }
            }

            SectionHeader {
                visible: page.place !== null && page.place.ended.length > 0
                //% "Ended"
                text: qsTrId("nanunana-place-ended")
            }

            Repeater {
                model: page.place !== null ? page.place.ended.length : 0

                WarningItem {
                    objectName: "endedRow"
                    warning: page.past(index)
                    ended: true
                    nowMs: page.nowMs
                    onClicked: pageStack.push(Qt.resolvedUrl("WarningPage.qml"), { "warningId": warning.id })
                }
            }

            Item {
                width: parent.width
                height: page.calm ? flick.height / 2 : 0
            }

            Unofficial {
                objectName: "placeFooter"
                text: page.footerText()
            }
        }

        ViewPlaceholder {
            objectName: "placeClear"
            enabled: page.calm
            //% "No warnings"
            text: qsTrId("nanunana-no-warnings")
            hintText: {
                if (!page.calm || page.place.updatedMs <= 0)
                    return ""
                //% "Checked at %1. Next check at about %2."
                return qsTrId("nanunana-place-clear-hint").arg(Words.clock(page.place.updatedMs)).arg(page.nextCheck())
            }
        }

        ViewPlaceholder {
            enabled: page.place === null
            //% "This place is no longer watched"
            text: qsTrId("nanunana-place-gone")
        }

        VerticalScrollDecorator {}
    }
}
