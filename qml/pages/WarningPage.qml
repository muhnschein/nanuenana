// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"
import "../components/Words.js" as Words

Page {
    id: page

    property string warningId
    readonly property var warning: { App.stateJson; return JSON.parse(App.warningJson(warningId)) }
    readonly property double nowMs: { App.stateJson; return Date.now() }
    readonly property bool cancelled: warning !== null && warning.kind === "cancel"

    function more() {
        if (warning === null)
            return ""
        var parts = []
        if (warning.contact.length > 0)
            parts.push(warning.contact)
        if (warning.web.length > 0)
            parts.push(warning.web)
        return parts.join("\n")
    }

    allowedOrientations: Orientation.All

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height + Theme.paddingLarge

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
            visible: page.warning !== null

            PageHeader {
                objectName: "warningHeader"
                title: page.warning !== null ? Words.categoryName(page.warning.category) : ""
                description: page.warning !== null ? page.warning.place : ""
            }

            Row {
                x: Theme.horizontalPageMargin
                height: Math.max(bars.height, words.height) + Theme.paddingMedium
                spacing: Theme.paddingMedium
                visible: page.warning !== null && !page.cancelled

                LevelBars {
                    id: bars

                    anchors.verticalCenter: parent.verticalCenter
                    big: true
                    level: page.warning !== null ? page.warning.level : 0
                }

                Column {
                    id: words

                    anchors.verticalCenter: parent.verticalCenter

                    Label {
                        objectName: "warningLevel"
                        text: page.warning !== null ? Words.levelName(page.warning.level) : ""
                        color: bars.lit.length > 0 ? bars.lit : Theme.primaryColor
                        font.pixelSize: Theme.fontSizeLarge
                    }

                    Label {
                        //% "Level %1 of 4"
                        text: page.warning !== null ? qsTrId("nanunana-warning-level-of").arg(page.warning.level) : ""
                        color: Theme.secondaryColor
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }

            Label {
                x: Theme.horizontalPageMargin
                visible: page.cancelled
                //% "All-clear"
                text: qsTrId("nanunana-all-clear")
                color: Theme.highlightColor
                font.pixelSize: Theme.fontSizeLarge
            }

            Label {
                objectName: "warningTitle"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                height: implicitHeight + Theme.paddingMedium
                text: page.warning !== null ? page.warning.headline : ""
                font.pixelSize: Theme.fontSizeLarge
                wrapMode: Text.Wrap
            }

            DetailItem {
                visible: value.length > 0
                //% "Issued by"
                label: qsTrId("nanunana-warning-issuer")
                value: page.warning !== null ? page.warning.sender : ""
            }

            DetailItem {
                visible: page.warning !== null && page.warning.sentMs > 0
                //% "Sent"
                label: qsTrId("nanunana-warning-sent")
                value: visible ? Words.whenLong(page.warning.sentMs, page.nowMs) : ""
            }

            DetailItem {
                visible: page.warning !== null && page.warning.changedMs > 0 && page.warning.changedMs !== page.warning.sentMs
                //% "Last change"
                label: qsTrId("nanunana-warning-changed")
                value: visible ? Words.whenLong(page.warning.changedMs, page.nowMs) : ""
            }

            DetailItem {
                visible: page.warning !== null && page.warning.expiresMs > 0
                //% "Valid until"
                label: qsTrId("nanunana-warning-expires")
                value: visible ? Words.whenLong(page.warning.expiresMs, page.nowMs) : ""
            }

            SectionHeader {
                visible: description.visible
                //% "Warning"
                text: qsTrId("nanunana-warning-section-text")
            }

            Label {
                id: description

                objectName: "warningDescription"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                visible: text.length > 0
                text: page.warning !== null ? page.warning.description : ""
                wrapMode: Text.Wrap
            }

            SectionHeader {
                visible: instruction.visible
                //% "What to do"
                text: qsTrId("nanunana-warning-section-instruction")
            }

            Label {
                id: instruction

                objectName: "warningInstruction"
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                visible: text.length > 0
                text: page.warning !== null ? page.warning.instruction : ""
                wrapMode: Text.Wrap
            }

            SectionHeader {
                visible: area.visible
                //% "Affected area"
                text: qsTrId("nanunana-warning-section-area")
            }

            Label {
                id: area

                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                visible: text.length > 0
                text: page.warning !== null ? page.warning.area : ""
                wrapMode: Text.Wrap
            }

            SectionHeader {
                visible: links.visible
                //% "More information"
                text: qsTrId("nanunana-warning-section-more")
            }

            LinkedLabel {
                id: links

                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                visible: plainText.length > 0
                plainText: page.more()
            }

            Item {
                width: 1
                height: Theme.paddingLarge
            }

            NoteBox {
                objectName: "warningNote"
                text: {
                    if (page.warning === null)
                        return ""
                    var seen = Words.when(page.warning.updatedMs, page.nowMs)
                    if (!page.warning.detailed) {
                        //% "The full text could not be loaded yet. Nanünana is unofficial and can be late: last successful update %1. Follow sirens, cell broadcast, radio and the authorities on site."
                        return qsTrId("nanunana-warning-note-brief").arg(seen)
                    }
                    //% "Shown unchanged as issued. Nanünana is unofficial and can be late: last successful update %1. Follow sirens, cell broadcast, radio and the authorities on site."
                    return qsTrId("nanunana-warning-note").arg(seen)
                }
            }
        }

        ViewPlaceholder {
            objectName: "warningGone"
            enabled: page.warning === null
            //% "This warning is no longer listed"
            text: qsTrId("nanunana-warning-gone")
            //% "It has ended or no longer concerns your places."
            hintText: qsTrId("nanunana-warning-gone-hint")
        }

        VerticalScrollDecorator {}
    }
}
