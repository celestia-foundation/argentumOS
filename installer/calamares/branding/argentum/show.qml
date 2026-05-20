// Calamares branding-API-2 slideshow.
// Self-contained: must not import installer/qml/components/ — Calamares'
// branding QML import path doesn't extend outside the branding directory.

import QtQuick 2.15

Item {
    id: root
    width: 800
    height: 480

    Rectangle {
        anchors.fill: parent
        color: "#0f1218"
    }

    property var messages: [
        "Installing argentumOS…",
        "Configuring Cinnamon…",
        "Almost there…"
    ]
    property int index: 0

    Text {
        id: label
        anchors.centerIn: parent
        text: root.messages[root.index]
        color: "#e6e8ee"
        font.pixelSize: 28
        font.family: "Sans"
        opacity: 1
        Behavior on opacity { NumberAnimation { duration: 400 } }
    }

    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: label.bottom
        anchors.topMargin: 24
        width: 120
        height: 3
        radius: 2
        color: "#7aa2f7"
    }

    Timer {
        interval: 3500
        running: true
        repeat: true
        onTriggered: {
            label.opacity = 0
            cycleTimer.start()
        }
    }
    Timer {
        id: cycleTimer
        interval: 450
        repeat: false
        onTriggered: {
            root.index = (root.index + 1) % root.messages.length
            label.opacity = 1
        }
    }

    // Calamares branding-API-2 hooks (no-ops here — we just animate freely).
    function onActivate()   {}
    function onLeave()      {}
}
