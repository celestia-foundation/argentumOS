import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../components"

Item {
    id: page
    anchors.fill: parent

    Rectangle {
        anchors.fill: parent
        color: "#0f1218"
    }

    function gsValue(key, fallback) {
        if (typeof Calamares !== "undefined"
            && Calamares.GlobalStorage
            && Calamares.GlobalStorage.contains(key)) {
            return Calamares.GlobalStorage.value(key)
        }
        return fallback
    }

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 18
        width: Math.min(parent.width * 0.7, 640)

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: "Review your choices"
            color: "#e6e8ee"
            font.pixelSize: 28
            font.bold: true
        }

        GridLayout {
            columns: 2
            columnSpacing: 24
            rowSpacing: 10
            Layout.fillWidth: true

            Text { text: "User";       color: "#9aa0ad"; font.pixelSize: 14 }
            Text { text: page.gsValue("username", "—"); color: "#e6e8ee"; font.pixelSize: 14 }

            Text { text: "Hostname";   color: "#9aa0ad"; font.pixelSize: 14 }
            Text { text: page.gsValue("hostname", "argentumOS"); color: "#e6e8ee"; font.pixelSize: 14 }

            Text { text: "Time zone";  color: "#9aa0ad"; font.pixelSize: 14 }
            Text { text: page.gsValue("locationTZ", "UTC"); color: "#e6e8ee"; font.pixelSize: 14 }

            Text { text: "Keyboard";   color: "#9aa0ad"; font.pixelSize: 14 }
            Text { text: page.gsValue("keyboardLayout", "us"); color: "#e6e8ee"; font.pixelSize: 14 }

            Text { text: "Disk plan";  color: "#9aa0ad"; font.pixelSize: 14 }
            Text {
                text: page.gsValue("partitionLayout", "Configured in previous step")
                color: "#e6e8ee"
                font.pixelSize: 14
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        ArgentumButton {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 16
            Layout.preferredWidth: 220
            text: "Install argentumOS"
            onClicked: if (typeof viewManager !== "undefined") viewManager.next()
        }
    }
}
