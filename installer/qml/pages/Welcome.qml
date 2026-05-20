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

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 24
        width: Math.min(parent.width * 0.6, 560)

        Image {
            Layout.alignment: Qt.AlignHCenter
            source: "../../share/calamares/branding/argentum/splash.png"
            fillMode: Image.PreserveAspectFit
            sourceSize.width: 160
        }

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: "Welcome to argentumOS"
            color: "#e6e8ee"
            font.pixelSize: 36
            font.bold: true
        }

        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: parent.width
            text: "Let's get you set up. This installer will guide you through " +
                  "language, keyboard, disk layout, and your user account."
            color: "#9aa0ad"
            font.pixelSize: 15
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }

        // TODO: future app-store onboarding screen is injected here, between
        // Welcome and Locale. Track in roadmap.

        ArgentumButton {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: 200
            text: "Begin"
            onClicked: if (typeof viewManager !== "undefined") viewManager.next()
        }
    }
}
