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
        width: Math.min(parent.width * 0.6, 520)

        Image {
            Layout.alignment: Qt.AlignHCenter
            source: "../../share/calamares/branding/argentum/splash.png"
            fillMode: Image.PreserveAspectFit
            sourceSize.width: 140
        }

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: "argentumOS is ready."
            color: "#e6e8ee"
            font.pixelSize: 30
            font.bold: true
        }

        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: parent.width
            text: "Remove the installation media and restart to use your new system."
            color: "#9aa0ad"
            font.pixelSize: 14
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }

        ArgentumButton {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: 200
            text: "Restart now"
            onClicked: {
                if (typeof Calamares !== "undefined"
                    && Calamares.JobQueue
                    && Calamares.JobQueue.singletonInstance) {
                    Calamares.JobQueue.singletonInstance().restartRequested()
                }
            }
        }
    }
}
