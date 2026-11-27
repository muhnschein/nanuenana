// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Nanunana 1.0
import "../../qml/components"
import "Probe.js" as Probe

Item {
    id: test

    property bool reachedEnd
    property bool scenarioRunning: true
    property int at
    property int waits
    property int checks
    property var checker: null
    property bool quietOver
    property var firstFix: null
    readonly property string snow: "dwd.2.49.0.0.276.0.DWD.PVW.1795700000000.snow"
    readonly property var steps: [
        startWithLocationOff, checkedWithoutPosition, locationOffAsksNoPositionAfterFixedWait,
        startWithLocationOn, positionArrives, checkedAtPosition, askedAgainOnNextCheck,
        startWithoutPositionSource, checkedAfterGuard, switchedOffWhileWaiting
    ]

    function view() {
        return JSON.parse(App.stateJson)
    }

    function positions() {
        var list = checker.resources
        for (var i = 0; i < list.length; ++i) {
            if (list[i].objectName === "positions")
                return list[i]
        }
        return null
    }

    function tick() {
        if (App.checking)
            return
        if (at >= steps.length) {
            reachedEnd = true
            scenarioRunning = false
            return
        }
        if (steps[at]() === false && waits < 1500) {
            waits += 1
            return
        }
        Probe.expect(waits < 1500, "step " + at + " never became ready")
        waits = 0
        at += 1
    }

    function startWithLocationOff() {
        App.acceptNotice()
        test.checker = checkerHost.createObject(test)
        Probe.expect(App.checking, "location off at start: check at once")
    }

    function checkedWithoutPosition() {
        if (test.checks < 1)
            return false
        Probe.expect(view().location === "off", "location off: " + view().location)
        positions().nmeaSource = Qt.resolvedUrl("positions/munich.nmea")
        Probe.expect(positions().valid, "position source ready")
        quiet.start()
    }

    function locationOffAsksNoPositionAfterFixedWait() {
        if (!test.quietOver)
            return false
        Probe.expect(!positions().position.latitudeValid, "location off: no position asked")
        Probe.expect(test.checks === 1, "location off: one check: " + test.checks)
        test.checker.destroy()
        test.checker = null
    }

    function startWithLocationOn() {
        if (test.checker !== null)
            return false
        Probe.expect(App.setSetting("location", "true"), "current location set on before start")
        test.checker = checkerHost.createObject(test)
        Probe.expect(!App.checking && test.checks === 1, "location on at start: no check before a position")
        Probe.expect(view().location === "waiting", "location on at start: waiting")
        var source = positions()
        if (!Probe.expect(source !== null && !source.valid, "no position source at start"))
            return
        source.nmeaSource = Qt.resolvedUrl("positions/munich.nmea")
        Probe.expect(source.valid, "position source ready")
    }

    function positionArrives() {
        return test.checks > 1
    }

    function checkedAtPosition() {
        var shown = view()
        Probe.expect(test.checks === 2, "one check: " + test.checks)
        Probe.expect(shown.location === "on", "the first check had a position: " + shown.location)
        var here = null
        for (var i = 0; i < shown.places.length; ++i) {
            if (shown.places[i].current)
                here = shown.places[i]
        }
        if (!Probe.expect(here !== null, "the current location is listed"))
            return
        var ids = here.warnings.map(function (w) { return w.id })
        Probe.expect(ids.indexOf(test.snow) >= 0, "the current location lists München's snow warning: " + ids)
        test.firstFix = positions().position.timestamp
        App.requestCheck()
        Probe.expect(!App.checking, "next check waits for a position")
    }

    function askedAgainOnNextCheck() {
        if (test.checks < 3)
            return false
        var again = positions().position.timestamp
        Probe.expect(again.getTime() !== test.firstFix.getTime(), "next check asked for a new position: " + again)
        test.checker.destroy()
        test.checker = null
    }

    function startWithoutPositionSource() {
        if (test.checker !== null)
            return false
        test.checker = checkerHost.createObject(test)
        Probe.expect(!positions().valid, "no position source")
        Probe.expect(!App.checking && test.checks === 3, "no position source: no check at once")
    }

    function checkedAfterGuard() {
        if (test.checks < 4)
            return false
        App.requestCheck()
        Probe.expect(!App.checking, "waiting for a position again")
        Probe.expect(App.setSetting("location", "false"), "current location set off")
        Probe.expect(App.checking, "switched off while waiting: check at once")
    }

    function switchedOffWhileWaiting() {
        return test.checks > 4
    }

    Component {
        id: checkerHost

        Checker {
        }
    }

    Timer {
        id: quiet

        interval: 1000
        onTriggered: test.quietOver = true
    }

    Connections {
        target: App
        onChecked: test.checks += 1
    }

    Timer {
        interval: 20
        repeat: true
        running: test.scenarioRunning
        onTriggered: test.tick()
    }
}
