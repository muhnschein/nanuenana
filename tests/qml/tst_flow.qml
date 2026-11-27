// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nanunana 1.0
import "../../qml/cover"
import "Probe.js" as Probe

ApplicationWindow {
    id: test

    property bool reachedEnd
    property bool scenarioRunning: true
    property int at
    property int waits
    property var places: null
    property var lastNotices: null
    property var menuRow: null
    property var rowMenu: null
    readonly property string fire: "mow.DE-BY-M-SE089-20261126-89-001"
    readonly property var steps: [
        openPlaces, acceptNotice, addMunich, munichListed, munichPage, fireWarning, coverWarned,
        addCologne, cologneStale, settingsPage, aboutPage, searchPage, searchHit, searchSaved, searchAgain,
        areaDialog, areaSaved, changeArea, areaChanged, areaIsDistrict, openRowMenu, removeFromMenu,
        removedAfterRemorse, removeAll
    ]

    function page(name) {
        return Qt.resolvedUrl("../../qml/pages/" + name + ".qml")
    }

    function view() {
        return JSON.parse(App.stateJson)
    }

    function settings() {
        return JSON.parse(App.settingsJson)
    }

    function iconOf(rows, headline) {
        for (var i = 0; i < rows.length; ++i) {
            if (Probe.text(rows[i], "warningHeadline") === headline)
                return String(Probe.one(rows[i], "categoryIcon").source).split("/").pop()
        }
        return "<no row " + headline + ">"
    }

    function marked(before, hit, after) {
        return before + "<font color=\"" + Theme.highlightColor + "\">" + hit + "</font>" + after
    }

    function placeNamed(name) {
        var list = view().places
        for (var i = 0; i < list.length; ++i) {
            if (list[i].name === name)
                return list[i]
        }
        return null
    }

    function backToPlaces() {
        pageStack.pop(test.places, PageStackAction.Immediate)
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
        Probe.expect(view().replay === false, "a start without the replay variable is no replay")
        Probe.expect(!view().noticeAccepted, "first start: notice not accepted")
        Probe.expect(view().places.length === 0, "first start: no places")
        Probe.expect(App.setSetting("language", JSON.stringify("de")), "warning language set")
        test.places = pageStack.push(page("PlacesPage"), {}, PageStackAction.Immediate)
    }

    function acceptNotice() {
        if (pageStack.depth < 2)
            return false
        var title = Probe.text(pageStack.currentPage, "noticeTitle")
        Probe.expect(title === qsTrId("nanunana-notice-title"), "notice title: " + title)
        pageStack.currentPage.accept()
    }

    function addMunich() {
        Probe.expect(view().noticeAccepted, "accepting the dialog records it")
        Probe.expect(pageStack.currentPage === test.places, "accepting returns to the places")
        Probe.expect(Probe.one(test.places, "placesEmpty").enabled, "no places: the hint shows")
        Probe.expect(Probe.one(test.places, "emptyUnofficial").visible, "the empty page says it is unofficial")
        Probe.expect(App.addPlace("München", "09162", 0) === "", "München is added")
        Probe.expect(App.addPlace("München", "09162", 0) === "duplicate", "a place is added once")
        Probe.expect(App.addPlace("Nirgendwo", "09162", 0) === "unknown", "unknown places are refused")
        Probe.expect(App.addPlace("München", "09162", 99) === "unknown", "areas over 25 km are refused")
        test.lastNotices = null
        App.check()
    }

    function munichListed() {
        Probe.expect(test.lastNotices !== null && test.lastNotices.length === 0, "the first check of a place alerts nothing")
        var place = placeNamed("München")
        if (!Probe.expect(place !== null && !place.stale, "München is up to date"))
            return
        Probe.expect(place.level === 3, "München's highest level is danger: " + place.level)
        Probe.expect(place.warnings.length >= 2 && place.ended.length === 1, "München lists warnings and one all-clear")
        Probe.expect(!Probe.one(test.places, "placesEmpty").enabled, "with a place the hint is gone")
        Probe.expect(!Probe.one(test.places, "emptyUnofficial").visible, "with a place the list's own line says it")
        Probe.expect(Probe.text(test.places, "placeName") === "München", "row name")
        Probe.expect(Probe.text(test.places, "placeHeadline") === "Rauchentwicklung durch Großbrand", "row headline: " + Probe.text(test.places, "placeHeadline"))
        Probe.expect(Probe.text(test.places, "placeArea") === qsTrId("nanunana-area-district"), "row area")
        Probe.expect(Probe.one(test.places, "placeLevel").level === 3, "row level")
        Probe.expect(!Probe.one(test.places, "testData").visible, "real warnings on the places carry no test-data box")
        pageStack.push(page("PlacePage"), { "placeId": place.id }, PageStackAction.Immediate)
    }

    function munichPage() {
        var shown = pageStack.currentPage
        var place = placeNamed("München")
        Probe.expect(Probe.one(shown, "placeHeader").title === "München", "place page title")
        var rows = Probe.named(shown, "warningRow")
        Probe.expect(rows.length === place.warnings.length, "one row per warning: " + rows.length)
        Probe.expect(Probe.named(shown, "endedRow").length === 1, "one ended row")
        var headlines = Probe.texts(shown, "warningHeadline")
        Probe.expect(headlines[0] === "Rauchentwicklung durch Großbrand", "the highest level comes first: " + headlines[0])
        Probe.expect(iconOf(rows, "Rauchentwicklung durch Großbrand") === "cat-civil.svg", "civil icon: " + iconOf(rows, "Rauchentwicklung durch Großbrand"))
        Probe.expect(iconOf(rows, "Amtliche WARNUNG vor SCHNEEFALL") === "cat-snow.svg", "a snow warning shows the snow icon: " + iconOf(rows, "Amtliche WARNUNG vor SCHNEEFALL"))
        Probe.expect(iconOf(rows, "Hochwasserinformation") === "cat-flood.svg", "flood icon: " + iconOf(rows, "Hochwasserinformation"))
        Probe.expect(iconOf(Probe.named(shown, "endedRow"), "Entwarnung: Bombenfund in Schwabing") === "all-clear.svg", "an ended warning shows the all-clear icon")
        var nameless = Qt.createComponent(Qt.resolvedUrl("../../qml/components/CategoryIcon.qml")).createObject(test)
        Probe.expect(String(nameless.source).split("/").pop() === "cat-civil.svg", "an icon without a name is the civil one: " + nameless.source)
        nameless.destroy()
        Probe.expect(!Probe.one(shown, "staleBox").visible, "an up-to-date place shows no stale box")
        Probe.expect(!Probe.one(shown, "testData").visible, "real warnings of a place carry no test-data box")
        Probe.expect(!Probe.one(shown, "placeClear").enabled, "a warned place is not called clear")
        Probe.expect(Probe.text(shown, "placeFooter").length > 0, "the footer says it is unofficial")
        pageStack.push(page("WarningPage"), { "warningId": test.fire }, PageStackAction.Immediate)
    }

    function fireWarning() {
        var shown = pageStack.currentPage
        Probe.expect(Probe.text(shown, "warningTitle") === "Rauchentwicklung durch Großbrand", "warning title")
        Probe.expect(Probe.text(shown, "warningLevel") === qsTrId("nanunana-level-danger"), "warning level: " + Probe.text(shown, "warningLevel"))
        Probe.expect(Probe.one(shown, "warningHeader").title === qsTrId("nanunana-category-civil"), "warning category")
        var description = Probe.text(shown, "warningDescription")
        Probe.expect(description.indexOf("Lagerhalle") >= 0 && description.indexOf("<") < 0, "description as plain text: " + description)
        Probe.expect(Probe.text(shown, "warningInstruction").indexOf("Fenster") >= 0, "instruction shown")
        Probe.expect(!Probe.one(shown, "warningGone").enabled, "a listed warning is not gone")
        Probe.expect(Probe.text(Probe.one(shown, "warningNote"), "noteText").length > 0, "the unofficial note shows")
        Probe.expect(!Probe.one(shown, "testData").visible, "a real warning carries no test-data box")
        backToPlaces()
        pageStack.push(coverHost, {}, PageStackAction.Immediate)
    }

    function coverWarned() {
        var shown = pageStack.currentPage
        Probe.expect(Probe.text(shown, "coverTitle") === qsTrId("nanunana-level-danger"), "cover level: " + Probe.text(shown, "coverTitle"))
        Probe.expect(Probe.text(shown, "coverFirst") === "München", "cover place")
        Probe.expect(Probe.text(shown, "coverSecond") === "Rauchentwicklung durch Großbrand", "cover headline")
        Probe.expect(!Probe.one(shown, "coverTestData").visible, "the cover of real warnings carries no test-data line")
        var notifier = Qt.createComponent(Qt.resolvedUrl("../../qml/components/Notifier.qml")).createObject(test)
        notifier.announce([{ "kind": "new", "level": 3, "place": "München", "id": test.fire, "headline": "Rauchentwicklung durch Großbrand" }])
        var note = notifier.notes[test.fire]
        Probe.expect(note && note.summary === qsTrId("nanunana-level-danger") + " · München", "a real notification is titled level and place: " + (note ? note.summary : "none"))
        notifier.destroy()
        backToPlaces()
    }

    function addCologne() {
        Probe.expect(App.addPlace("Köln", "05315", 0) === "", "Köln is added")
        test.lastNotices = null
        App.check()
    }

    function cologneStale() {
        var place = placeNamed("Köln")
        if (!Probe.expect(place !== null && place.stale, "Köln, with no answer, is stale"))
            return
        var headlines = Probe.texts(test.places, "placeHeadline")
        Probe.expect(headlines.length === 2 && headlines[1] === qsTrId("nanunana-places-never"), "the stale row says so: " + headlines)
        pageStack.push(page("PlacePage"), { "placeId": place.id }, PageStackAction.Immediate)
        var shown = pageStack.currentPage
        Probe.expect(Probe.one(shown, "staleBox").visible, "the stale place shows its box")
        Probe.expect(Probe.one(shown, "tryNow").visible, "and offers a retry")
        Probe.expect(!Probe.one(shown, "placeClear").enabled, "a stale place is never called clear")
        backToPlaces()
        pageStack.push(coverHost, {}, PageStackAction.Immediate)
        Probe.expect(Probe.text(pageStack.currentPage, "coverTitle") === qsTrId("nanunana-cover-stale"), "cover says not updated")
        backToPlaces()
        pageStack.push(page("SettingsPage"), {}, PageStackAction.Immediate)
    }

    function settingsPage() {
        var shown = pageStack.currentPage
        var location = Probe.one(shown, "location")
        Probe.expect(!location.checked, "current location is off by default")
        location.clicked(null)
        Probe.expect(settings().location && location.checked, "the switch turns it on")
        Probe.expect(view().location === "waiting", "on without a position: waiting")
        location.clicked(null)
        Probe.expect(!settings().location && view().location === "off", "and off again")
        var weather = Probe.one(shown, "threshold-weather")
        Probe.expect(weather.currentIndex === 1, "weather alerts from warning by default: " + weather.currentIndex)
        weather.chosen("danger")
        Probe.expect(settings().weather === "danger", "threshold saved")
        var interval = Probe.one(shown, "interval")
        interval.chosen(15)
        Probe.expect(settings().interval === 15, "interval saved")
        Probe.one(shown, "allClear").clicked(null)
        Probe.expect(!settings().allClear, "all-clear switched off")
        Probe.expect(!App.setSetting("interval", "7"), "odd intervals are refused")
        Probe.one(shown, "about").clicked(null)
    }

    function aboutPage() {
        Probe.expect(pageStack.depth === 3, "about opens from settings: depth " + pageStack.depth)
        backToPlaces()
        pageStack.push(page("AddPlacePage"), {}, PageStackAction.Immediate)
    }

    function searchPage() {
        var field = Probe.one(pageStack.currentPage, "placeSearch")
        Probe.expect(field.focus, "the search field starts focused")
        field.text = "Rehlingen-S"
    }

    function searchHit() {
        var shown = pageStack.currentPage
        Probe.one(shown, "hitList").forceLayout()
        if (Probe.named(shown, "hitRow").length !== 1)
            return false
        Probe.expect(Probe.text(shown, "hitDistrict") === "Landkreis Saarlouis, Saarland", "hit district: " + Probe.text(shown, "hitDistrict"))
        Probe.expect(Probe.one(shown, "placeSearch").focus, "hits arriving leave the focus in the search field")
        Probe.expect(Probe.one(shown, "hitList").currentIndex === -1, "no hit is made the current one")
        Probe.expect(Probe.text(shown, "hitName") === marked("", "Rehlingen-S", "iersburg"), "the typed part of the name is marked: " + Probe.text(shown, "hitName"))
        Probe.one(shown, "placeSearch").text = "munch"
    }

    function searchSaved() {
        var shown = pageStack.currentPage
        Probe.one(shown, "hitList").forceLayout()
        var districts = Probe.texts(shown, "hitDistrict")
        if (districts.length === 0 || districts[0] !== "Kreisfreie Stadt München, Bayern")
            return false
        Probe.expect(!Probe.named(shown, "hitRow")[0].enabled, "a saved place cannot be added again")
        Probe.expect(Probe.texts(shown, "hitName")[0] === marked("", "Münch", "en"), "typed without umlaut, the part with umlaut is marked: " + Probe.texts(shown, "hitName")[0])
        Probe.expect(Probe.one(shown, "placeSearch").focus, "changing hits leave the focus in the search field")
        Probe.one(shown, "placeSearch").text = "siersb"
    }

    function searchAgain() {
        var shown = pageStack.currentPage
        Probe.one(shown, "hitList").forceLayout()
        var districts = Probe.texts(shown, "hitDistrict")
        if (districts.length !== 1 || districts[0] !== "Landkreis Saarlouis, Saarland")
            return false
        Probe.expect(Probe.text(shown, "hitName") === marked("Rehlingen-", "Siersb", "urg"), "a later word of the name is marked where it stands: " + Probe.text(shown, "hitName"))
        Probe.one(shown, "hitRow").clicked(null)
    }

    function areaDialog() {
        var shown = pageStack.currentPage
        Probe.expect(Probe.text(shown, "areaName") === "Rehlingen-Siersburg", "area dialog names the place")
        Probe.expect(shown.chosenKm === 5, "5 km by default: " + shown.chosenKm)
        Probe.one(shown, "areaDistance").value = 7
        Probe.expect(shown.chosenKm === 7, "the slider sets the distance")
        Probe.expect(Probe.text(shown, "areaExplanation").indexOf("7 km") >= 0, "the explanation follows")
        shown.accept()
    }

    function areaSaved() {
        Probe.one(test.places, "placeList").positionViewAtEnd()
        if (Probe.texts(test.places, "placeArea").indexOf(qsTrId("nanunana-area-km").arg(7)) < 0)
            return false
        Probe.expect(pageStack.currentPage === test.places, "saving returns to the places")
        var place = placeNamed("Rehlingen-Siersburg")
        Probe.expect(place !== null && place.km === 7, "the place is saved with its distance")
    }

    function changeArea() {
        var place = placeNamed("Rehlingen-Siersburg")
        pageStack.push(page("AreaDialog"), {
            "placeId": place.id,
            "name": place.name,
            "districtName": place.districtName,
            "stateName": place.stateName,
            "km": place.km
        }, PageStackAction.Immediate)
    }

    function areaChanged() {
        var shown = pageStack.currentPage
        Probe.expect(shown.chosenKm === 7, "the dialog opens with the saved distance: " + shown.chosenKm)
        Probe.one(shown, "areaMode").currentIndex = 1
        Probe.expect(shown.chosenKm === 0, "whole district chosen: " + shown.chosenKm)
        shown.accept()
    }

    function areaIsDistrict() {
        var place = placeNamed("Rehlingen-Siersburg")
        Probe.expect(place !== null && place.km === 0, "the changed area is saved")
        Probe.expect(view().places.length === 3, "changing an area adds no place")
    }

    function openRowMenu() {
        Probe.one(test.places, "placeList").positionViewAtBeginning()
        var rows = Probe.named(test.places, "placeRow")
        for (var i = 0; i < rows.length; ++i) {
            if (rows[i].place !== null && rows[i].place.name === "München")
                test.menuRow = rows[i]
        }
        if (test.menuRow === null)
            return false
        test.rowMenu = test.menuRow.openMenu()
    }

    function removeFromMenu() {
        Probe.one(test.rowMenu, "removePlace").clicked()
        test.menuRow.closeMenu()
    }

    function removedAfterRemorse() {
        if (placeNamed("München") !== null)
            return false
        Probe.expect(view().places.length === 2, "only the removed place is gone")
    }

    function removeAll() {
        var list = view().places
        for (var i = 0; i < list.length; ++i)
            Probe.expect(App.removePlace(list[i].id), "remove " + list[i].name)
        Probe.expect(view().places.length === 0, "all places removed")
        Probe.expect(Probe.one(test.places, "placesEmpty").enabled, "the hint is back")
    }

    Component {
        id: coverHost

        Page {
            CoverPage {
                anchors.fill: parent
            }
        }
    }

    Connections {
        target: App
        onChecked: test.lastNotices = JSON.parse(notices)
    }

    Timer {
        interval: 20
        repeat: true
        running: test.scenarioRunning
        onTriggered: test.tick()
    }
}
