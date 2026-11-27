// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick 2.6
import Nemo.DBus 2.0

DBusAdaptor {
    id: taps

    signal warningRequested(string warningId)
    signal placesRequested()

    service: "nanunana.nanunana"
    path: "/"
    iface: "nanunana.nanunana"
    xml: "  <interface name=\"nanunana.nanunana\">\n" +
         "    <method name=\"showWarning\">\n" +
         "      <arg name=\"id\" type=\"s\" direction=\"in\"/>\n" +
         "    </method>\n" +
         "    <method name=\"showPlaces\"/>\n" +
         "  </interface>\n"

    function showWarning(warningId) {
        taps.warningRequested(warningId)
    }

    function showPlaces() {
        taps.placesRequested()
    }
}
