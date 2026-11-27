// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../components"
import "../components/Words.js" as Words

Page {
    id: page

    readonly property var settings: JSON.parse(App.settingsJson)
    readonly property var thresholds: ["information", "warning", "danger", "extreme", "never"]
    readonly property var thresholdNames: [
        //% "Information and higher"
        qsTrId("nanunana-settings-from-information"),
        //% "Warning and higher"
        qsTrId("nanunana-settings-from-warning"),
        //% "Danger and higher"
        qsTrId("nanunana-settings-from-danger"),
        //% "Extreme danger only"
        qsTrId("nanunana-settings-from-extreme"),
        //% "No alerts"
        qsTrId("nanunana-settings-never")
    ]
    readonly property var languages: ["", "de", "en", "fr", "es", "pl", "tr", "ru", "uk", "ar"]
    readonly property var languageNames: [
        //% "Phone language"
        qsTrId("nanunana-settings-language-phone"),
        "Deutsch", "English", "Français", "Español", "Polski", "Türkçe", "Русский", "Українська", "العربية"
    ]

    function set(key, value) {
        App.setSetting(key, JSON.stringify(value))
    }

    function value(key, fallback) {
        return settings !== null && settings[key] !== undefined ? settings[key] : fallback
    }

    allowedOrientations: Orientation.All

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height + Theme.paddingLarge

        Column {
            id: column

            width: parent.width

            PageHeader {
                //% "Settings"
                title: qsTrId("nanunana-settings-title")
            }

            SectionHeader {
                //% "Alert me about"
                text: qsTrId("nanunana-settings-alerts")
            }

            Repeater {
                model: ["civil", "police", "weather", "flood"]

                ChoiceBox {
                    objectName: "threshold-" + modelData
                    label: Words.categoryName(modelData)
                    values: page.thresholds
                    labels: page.thresholdNames
                    value: page.value(modelData, "information")
                    //% "Lower levels still show in the app."
                    description: index === 3 ? qsTrId("nanunana-settings-alerts-hint") : ""
                    onChosen: page.set(modelData, value)
                }
            }

            TextSwitch {
                objectName: "allClear"
                automaticCheck: false
                checked: page.value("allClear", true)
                //% "All-clear"
                text: qsTrId("nanunana-all-clear")
                //% "Tell me when a warning I was alerted about has ended."
                description: qsTrId("nanunana-settings-all-clear-hint")
                onClicked: page.set("allClear", !checked)
            }

            TextSwitch {
                objectName: "everyUpdate"
                automaticCheck: false
                checked: page.value("everyUpdate", false)
                //% "Every update"
                text: qsTrId("nanunana-settings-every-update")
                //% "Off: a changed warning alerts again only when its level rises."
                description: qsTrId("nanunana-settings-every-update-hint")
                onClicked: page.set("everyUpdate", !checked)
            }

            SectionHeader {
                //% "Current location"
                text: qsTrId("nanunana-area-current")
            }

            TextSwitch {
                objectName: "location"
                automaticCheck: false
                checked: page.value("location", false)
                //% "Warnings at your location"
                text: qsTrId("nanunana-settings-location")
                //% "Shown first on the home page, with alerts like a saved place. Your position stays on this phone. Needs positioning turned on in the phone settings."
                description: qsTrId("nanunana-settings-location-hint")
                onClicked: page.set("location", !checked)
            }

            SectionHeader {
                //% "Checking for warnings"
                text: qsTrId("nanunana-settings-checking")
            }

            ChoiceBox {
                objectName: "interval"
                //% "Check every"
                label: qsTrId("nanunana-settings-interval")
                values: [5, 10, 15, 30]
                labels: values.map(function (minutes) {
                    //% "%n minutes"
                    return qsTrId("nanunana-settings-minutes", minutes)
                })
                value: page.value("interval", 5)
                //% "Also wakes the phone from sleep. Longer intervals use less battery and are later."
                description: qsTrId("nanunana-settings-interval-hint")
                onChosen: page.set("interval", value)
            }

            NoteBox {
                //% "Nanünana checks only while it is open or minimised. If you close it, you get no alerts. Sailfish OS offers apps no other way."
                text: qsTrId("nanunana-settings-open-only")
            }

            TextSwitch {
                objectName: "notifyFailure"
                automaticCheck: false
                checked: page.value("notifyFailure", true)
                //% "Tell me when checking fails"
                text: qsTrId("nanunana-settings-failure")
                //% "One notification after 30 minutes without a successful check."
                description: qsTrId("nanunana-settings-failure-hint")
                onClicked: page.set("notifyFailure", !checked)
            }

            SectionHeader {
                //% "Warnings"
                text: qsTrId("nanunana-settings-warnings")
            }

            ChoiceBox {
                objectName: "language"
                //% "Language"
                label: qsTrId("nanunana-settings-language")
                values: page.languages
                labels: page.languageNames
                value: page.value("language", "")
                //% "Only translations the issuer provides. Without one the original is shown."
                description: qsTrId("nanunana-settings-language-hint")
                onChosen: page.set("language", value)
            }

            SectionHeader {
                //% "About"
                text: qsTrId("nanunana-settings-about")
            }

            BackgroundItem {
                objectName: "about"
                width: parent.width
                height: Theme.itemSizeMedium
                onClicked: pageStack.push(Qt.resolvedUrl("AboutPage.qml"))

                Column {
                    anchors {
                        left: parent.left
                        leftMargin: Theme.horizontalPageMargin
                        right: parent.right
                        rightMargin: Theme.horizontalPageMargin
                        verticalCenter: parent.verticalCenter
                    }

                    Label {
                        width: parent.width
                        //% "About Nanünana"
                        text: qsTrId("nanunana-about-row")
                        color: parent.parent.highlighted ? Theme.highlightColor : Theme.primaryColor
                        truncationMode: TruncationMode.Fade
                    }

                    Label {
                        width: parent.width
                        //% "Official channels, sources, licence"
                        text: qsTrId("nanunana-about-row-hint")
                        color: Theme.secondaryColor
                        font.pixelSize: Theme.fontSizeSmall
                        truncationMode: TruncationMode.Fade
                    }
                }
            }
        }

        VerticalScrollDecorator {}
    }
}
