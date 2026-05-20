import QtQuick 2.15
import QtQuick.Controls 2.15

ProgressBar {
    id: control
    implicitHeight: 6

    background: Rectangle {
        implicitHeight: 6
        radius: 3
        color: "#1a1f2b"
    }

    contentItem: Item {
        Rectangle {
            width: control.position * parent.width
            height: parent.height
            radius: 3
            color: "#7aa2f7"
        }
    }
}
