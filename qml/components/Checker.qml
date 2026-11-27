// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import QtPositioning 5.2
import Nemo.KeepAlive 1.2
import Nanunana 1.0

Item {
    id: checker

    readonly property var settings: JSON.parse(App.settingsJson)
    readonly property var view: JSON.parse(App.stateJson)
    readonly property bool allowed: view !== null && view.noticeAccepted
    readonly property bool locating: allowed && settings !== null && settings.location
    readonly property int minutes: settings !== null ? settings.interval : 5
    property bool waitingForFix

    function frequency() {
        switch (minutes) {
        case 30:
            return BackgroundJob.ThirtyMinutes
        case 15:
            return BackgroundJob.FifteenMinutes
        case 10:
            return BackgroundJob.TenMinutes
        default:
            return BackgroundJob.FiveMinutes
        }
    }

    function run() {
        if (!allowed || App.checking || waitingForFix)
            return false
        if (locating) {
            waitingForFix = true
            fixGuard.restart()
            ask()
        } else {
            App.check()
        }
        return true
    }

    function ask() {
        if (waitingForFix && source.valid)
            source.update()
    }

    function proceed() {
        if (!waitingForFix)
            return
        waitingForFix = false
        fixGuard.stop()
        App.check()
    }

    onLocatingChanged: locating ? run() : proceed()

    BackgroundJob {
        id: job

        enabled: checker.allowed
        frequency: checker.frequency()
        triggeredOnEnable: true
        onTriggered: {
            if (!checker.run() && !App.checking && !checker.waitingForFix)
                job.finished()
        }
    }

    PositionSource {
        id: source

        objectName: "positions"
        active: false
        onValidChanged: checker.ask()
        onSupportedPositioningMethodsChanged: checker.ask()
        onPositionChanged: {
            if (!position.latitudeValid || !position.longitudeValid)
                return
            App.setPosition(position.coordinate.latitude, position.coordinate.longitude)
            checker.proceed()
        }
        onSourceErrorChanged: {
            if (sourceError !== PositionSource.NoError)
                checker.proceed()
        }
    }

    Timer {
        id: fixGuard

        interval: 20000
        onTriggered: checker.proceed()
    }

    Connections {
        target: App
        onCheckRequested: checker.run()
        onChecked: {
            if (job.running)
                job.finished()
        }
    }
}
