// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../../../qml/cover"
import "../Probe.js" as Probe

ApplicationWindow {
    id: test

    property bool reachedEnd
    property bool scenarioRunning: true
    property int at
    property int waits
    property var places: null
    readonly property string fire: "mow.DE-BY-M-SE089-20261126-89-001"
    readonly property string title: "Test data"
    readonly property string words: "These are not real warnings."
    readonly property var steps: [openPlaces, acceptNotice, addMunich, placesSayTestData, placeSaysTestData, warningSaysTestData, coverSaysTestData]

    function page(name) {
        return Qt.resolvedUrl("../../../qml/pages/" + name + ".qml")
    }

    function view() {
        return JSON.parse(App.stateJson)
    }

    function boxSaysTestData(shown, where) {
        var box = Probe.one(shown, "testData")
        Probe.expect(box.visible && box.height > 0, where + ": the test-data box shows")
        Probe.expect(Probe.text(box, "noteTitle") === test.title, where + ": box title: " + Probe.text(box, "noteTitle"))
        Probe.expect(Probe.text(box, "noteText") === test.words, where + ": box text: " + Probe.text(box, "noteText"))
    }

    function tick() {
        if (pageStack.busy || App.checking)
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

    function openPlaces() {
        Probe.expect(view().replay === true, "a start with the replay variable is a replay")
        test.places = pageStack.push(page("PlacesPage"), {}, PageStackAction.Immediate)
    }

    function acceptNotice() {
        if (pageStack.depth < 2)
            return false
        pageStack.currentPage.accept()
    }

    function addMunich() {
        boxSaysTestData(test.places, "places without a place")
        Probe.expect(Probe.emptyTextTop(test.places) === "clear", "the empty text starts below the test-data box: " + Probe.emptyTextTop(test.places))
        Probe.expect(App.addPlace("München", "09162", 0) === "", "München is added")
        App.check()
    }

    function placesSayTestData() {
        var place = view().places[0]
        if (!Probe.expect(place.warnings.length > 0, "the replayed answers list warnings"))
            return
        boxSaysTestData(test.places, "places")
        pageStack.push(page("PlacePage"), { "placeId": place.id }, PageStackAction.Immediate)
    }

    function placeSaysTestData() {
        boxSaysTestData(pageStack.currentPage, "place")
        pageStack.push(page("WarningPage"), { "warningId": test.fire }, PageStackAction.Immediate)
    }

    function warningSaysTestData() {
        var shown = pageStack.currentPage
        Probe.expect(Probe.text(shown, "warningTitle") === "Rauchentwicklung durch Großbrand", "the replayed warning opens")
        boxSaysTestData(shown, "warning")
        pageStack.pop(test.places, PageStackAction.Immediate)
        pageStack.push(coverHost, {}, PageStackAction.Immediate)
    }

    function coverSaysTestData() {
        var line = Probe.one(pageStack.currentPage, "coverTestData")
        Probe.expect(line.visible && String(line.text) === test.title, "the cover says test data first: " + line.text)
        var notifier = Qt.createComponent(Qt.resolvedUrl("../../../qml/components/Notifier.qml")).createObject(test)
        notifier.announce([
            { "kind": "new", "level": 3, "place": "München", "id": test.fire, "headline": "Rauchentwicklung durch Großbrand" },
            { "kind": "failure", "level": 0, "place": "", "id": "", "headline": "" }
        ])
        var note = notifier.notes[test.fire]
        var expected = test.title + " · " + qsTrId("nanunana-level-danger") + " · München"
        Probe.expect(note && note.summary === expected, "a replayed notification says test data first: " + (note ? note.summary : "none"))
        var failure = notifier.notes["failure"]
        Probe.expect(failure && failure.summary === test.title + " · " + qsTrId("nanunana-note-failure-title"), "a replayed failure notification says test data first: " + (failure ? failure.summary : "none"))
        notifier.destroy()
        pageStack.pop(test.places, PageStackAction.Immediate)
    }

    Component {
        id: coverHost

        Page {
            CoverPage {
                anchors.fill: parent
            }
        }
    }

    Timer {
        interval: 20
        repeat: true
        running: test.scenarioRunning
        onTriggered: test.tick()
    }
}
