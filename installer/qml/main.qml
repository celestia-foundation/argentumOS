// Standalone QML harness for iterating on the installer pages without
// booting a VM. Not consumed by Calamares — Calamares owns its own QMainWindow
// and loads our pages as view-step modules.
//
// Run: qmlscene installer/qml/main.qml

import QtQuick 2.15
import QtQuick.Controls 2.15

ApplicationWindow {
    id: window
    width: 1024
    height: 680
    visible: true
    flags: Qt.Window | Qt.FramelessWindowHint
    color: "#0f1218"
    title: "argentumOS installer (preview)"

    Loader {
        anchors.fill: parent
        source: "pages/Welcome.qml"
    }
}
