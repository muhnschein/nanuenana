// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "components"

ApplicationWindow {
    id: window

    function showWarning(warningId) {
        window.activate()
        if (App.warningJson(warningId) === "null")
            return
        pageStack.completeAnimation()
        pageStack.push(Qt.resolvedUrl("pages/WarningPage.qml"), { "warningId": warningId })
    }

    initialPage: Qt.resolvedUrl("pages/PlacesPage.qml")
    cover: Qt.resolvedUrl("cover/CoverPage.qml")
    allowedOrientations: defaultAllowedOrientations
    _defaultLabelFormat: Text.PlainText

    Checker {
        id: checker
    }

    Notifier {
        id: notifier
    }

    Taps {
        onWarningRequested: window.showWarning(warningId)
        onPlacesRequested: window.activate()
    }

    Connections {
        target: App
        onChecked: {
            var list = JSON.parse(notices)
            notifier.announce(list)
            var view = JSON.parse(App.stateJson)
            if (view !== null && !view.failing)
                notifier.clearFailure()
        }
    }

    Connections {
        target: Qt.application
        onStateChanged: {
            if (Qt.application.state === Qt.ApplicationActive)
                App.reload()
        }
    }
}
