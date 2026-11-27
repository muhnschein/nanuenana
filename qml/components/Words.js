// SPDX-License-Identifier: GPL-3.0-or-later
.pragma library
.import Sailfish.Silica 1.0 as Silica

var DAY = 86400000

function levelName(level) {
    switch (level) {
    case 4:
        //% "Extreme danger"
        return qsTrId("nanunana-level-extreme")
    case 3:
        //% "Danger"
        return qsTrId("nanunana-level-danger")
    case 2:
        //% "Warning"
        return qsTrId("nanunana-level-warning")
    case 1:
        //% "Information"
        return qsTrId("nanunana-level-information")
    default:
        return ""
    }
}

function categoryName(category) {
    switch (category) {
    case "civil":
        //% "Civil protection"
        return qsTrId("nanunana-category-civil")
    case "police":
        //% "Police"
        return qsTrId("nanunana-category-police")
    case "weather":
        //% "Weather"
        return qsTrId("nanunana-category-weather")
    case "flood":
        //% "Flood"
        return qsTrId("nanunana-category-flood")
    default:
        return ""
    }
}

function levelColor(level, lightOnDark) {
    var dark = ["", "#ffdd55", "#ffa94d", "#ff7a6e", "#e3a0ff"]
    var light = ["", "#6b5600", "#9a4a00", "#b3261e", "#7b1fa2"]
    var colors = lightOnDark ? dark : light
    return level >= 1 && level <= 4 ? colors[level] : ""
}

function marked(text, start, length, color) {
    if (length <= 0)
        return text
    var end = start + length
    return text.substring(0, start) + "<font color=\"" + color + "\">" + text.substring(start, end) + "</font>" + text.substring(end)
}

function clock(ms) {
    return Silica.Format.formatDate(new Date(ms), Silica.Formatter.TimeValue)
}

function sameDay(a, b) {
    return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate()
}

function when(ms, nowMs) {
    var date = new Date(ms)
    var now = new Date(nowMs)
    if (sameDay(date, now))
        return clock(ms)
    if (Math.abs(nowMs - ms) < 6 * DAY)
        return Qt.formatDate(date, "ddd") + " " + clock(ms)
    return Silica.Format.formatDate(date, Silica.Formatter.DateMedium) + " " + clock(ms)
}

function whenLong(ms, nowMs) {
    var date = new Date(ms)
    if (sameDay(date, new Date(nowMs))) {
        //% "Today %1"
        return qsTrId("nanunana-time-today").arg(clock(ms))
    }
    return Silica.Format.formatDate(date, Silica.Formatter.DateMedium) + " " + clock(ms)
}

function areaText(place) {
    if (!place)
        return ""
    if (place.current) {
        //% "Current location"
        return qsTrId("nanunana-area-current")
    }
    if (place.km === 0) {
        //% "Whole district"
        return qsTrId("nanunana-area-district")
    }
    //% "%1 km"
    return qsTrId("nanunana-area-km").arg(place.km)
}

function noticeTitle(notice) {
    if (notice.kind === "failure") {
        //% "No updates for 30 minutes"
        return qsTrId("nanunana-note-failure-title")
    }
    //% "All-clear"
    var word = notice.kind === "ended" ? qsTrId("nanunana-all-clear") : levelName(notice.level)
    return notice.place.length > 0 ? word + " · " + notice.place : word
}

function noticeBody(notice) {
    if (notice.kind === "failure") {
        //% "Nanünana cannot reach the warning service. Warnings may be missing."
        return qsTrId("nanunana-note-failure-body")
    }
    return notice.headline
}

function placeOf(view, id) {
    var places = view && view.places ? view.places : []
    for (var i = 0; i < places.length; ++i) {
        if (places[i].id === id)
            return places[i]
    }
    return null
}

function summary(view) {
    var places = view && view.places ? view.places : []
    var result = { "places": places.length, "warnings": 0, "warned": 0, "level": 0, "top": null, "stale": null, "checkedMs": view ? view.checkedMs : 0 }
    for (var i = 0; i < places.length; ++i) {
        var place = places[i]
        result.warnings += place.warnings.length
        if (place.warnings.length > 0)
            result.warned += 1
        if (place.level > result.level) {
            result.level = place.level
            result.top = place
        }
        if (place.stale && !result.stale)
            result.stale = place
    }
    return result
}
