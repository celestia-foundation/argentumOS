import QtQuick 2.15
import QtQuick.Controls 2.15

Button {
    id: root
    padding: 12
    leftPadding: 18
    rightPadding: 18

    background: Rectangle {
        radius: 6
        color: root.pressed ? "#5e7cc4"
             : root.hovered ? "#8ab4ff"
             : "#7aa2f7"
        border.width: 1
        border.color: "#0f1218"
    }

    contentItem: Text {
        text: root.text
        color: root.enabled ? "#0f1218" : "#6c7280"
        font.pixelSize: 14
        font.bold: true
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
}
