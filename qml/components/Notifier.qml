// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Nemo.Notifications 1.0
import Nanunana 1.0
import "Words.js" as Words

Item {
    id: notifier

    property var notes: ({})
    readonly property string bus: "nanunana.nanunana"

    function action(notice) {
        var warning = notice.kind !== "failure"
        return [{
            "name": "default",
            "service": notifier.bus,
            "path": "/",
            "iface": notifier.bus,
            "method": warning ? "showWarning" : "showPlaces",
            "arguments": warning ? [notice.id] : []
        }]
    }

    function announce(notices) {
        var replay = Words.replaying(JSON.parse(App.stateJson))
        for (var i = 0; i < notices.length; ++i)
            notifier.publish(notices[i], replay)
    }

    function publish(notice, replay) {
        var key = notice.kind === "failure" ? "failure" : notice.id
        var note = notifier.notes[key]
        if (!note) {
            note = noteComponent.createObject(notifier)
            if (!note)
                return
            notifier.notes[key] = note
        }
        note.summary = Words.noticeTitle(notice, replay)
        note.body = Words.noticeBody(notice)
        note.previewSummary = note.summary
        note.previewBody = note.body
        note.urgency = notice.kind !== "ended" && notice.level >= 3 ? Notification.Critical : Notification.Normal
        note.timestamp = new Date()
        note.remoteActions = notifier.action(notice)
        note.publish()
    }

    function clearFailure() {
        var note = notifier.notes["failure"]
        if (note)
            note.close()
    }

    Component {
        id: noteComponent

        Notification {
            //% "Nanünana"
            appName: qsTrId("nanunana-app-name")
            appIcon: "harbour-nanunana"
        }
    }
}
