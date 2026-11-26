// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0

Page {
    id: page

    readonly property var sections: [
        {
            //% "Not an official app"
            "title": qsTrId("nanunana-about-unofficial"),
            //% "Volunteers make Nanünana. No authority runs, checks or endorses it. It can be late and it can miss a warning. In an emergency follow the official channels first."
            "text": qsTrId("nanunana-about-unofficial-text")
        },
        {
            //% "Official channels in Germany"
            "title": qsTrId("nanunana-about-official"),
            //% "Cell broadcast on your phone, sirens, radio and television.\nhttps://warnung.bund.de\nEmergency calls: 112 (fire and rescue), 110 (police)."
            "text": qsTrId("nanunana-about-official-text")
        },
        {
            //% "Where the warnings come from"
            "title": qsTrId("nanunana-about-sources"),
            //% "Bundesamt für Bevölkerungsschutz und Katastrophenhilfe (warnung.bund.de), with warnings of the civil protection authorities of the Länder and districts, the police forces of the Länder, Deutscher Wetterdienst and the Länderübergreifendes Hochwasserportal. Texts are shown unchanged.\n\nPlace names: GeoNames, CC BY 4.0, https://www.geonames.org"
            "text": qsTrId("nanunana-about-sources-text")
        },
        {
            //% "Where Nanünana is offered"
            "title": qsTrId("nanunana-about-offered"),
            //% "Germany. More countries follow one by one.\n\nNanünana is not offered in the Russian Federation, in Belarus, or in territories under Russian control outside Russia. It is not offered in Iran, North Korea or Syria."
            "text": qsTrId("nanunana-about-offered-text")
        },
        {
            //% "Your data"
            "title": qsTrId("nanunana-about-data"),
            //% "Places, settings and your position stay on this phone. Nanünana asks warnung.bund.de for the warnings of your districts; that service sees the district, not the place or your position. No account, no tracking."
            "text": qsTrId("nanunana-about-data-text")
        },
        {
            //% "Licence"
            "title": qsTrId("nanunana-about-licence"),
            //% "Free software, GPL-3.0-or-later.\nhttps://github.com/muhnschein/nan-nana"
            "text": qsTrId("nanunana-about-licence-text")
        }
    ]

    allowedOrientations: Orientation.All

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: column.height + Theme.paddingLarge

        Column {
            id: column

            width: parent.width

            PageHeader {
                //% "Nanünana"
                title: qsTrId("nanunana-app-name")
                //% "Version %1"
                description: qsTrId("nanunana-about-version").arg(App.version)
            }

            Label {
                x: Theme.horizontalPageMargin
                width: parent.width - 2 * x
                //% "An unofficial app that shows official warnings for places you choose and for where you are."
                text: qsTrId("nanunana-about-summary")
                wrapMode: Text.Wrap
            }

            Repeater {
                model: page.sections

                Column {
                    width: column.width

                    SectionHeader {
                        text: modelData.title
                    }

                    LinkedLabel {
                        x: Theme.horizontalPageMargin
                        width: parent.width - 2 * x
                        plainText: modelData.text
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }
        }

        VerticalScrollDecorator {}
    }
}
