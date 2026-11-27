// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6

QtObject {
    property bool reachedEnd
    Component.onCompleted: {
        console.error("FAILED: the canary reports a failure")
        reachedEnd = true
    }
}
