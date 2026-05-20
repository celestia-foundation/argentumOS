import QtQuick 2.15
import QtQuick.Controls 2.15

TextField {
    color: "#e6e8ee"
    placeholderTextColor: "#6c7280"
    selectionColor: "#7aa2f7"
    selectedTextColor: "#0f1218"
    padding: 8

    background: Rectangle {
        radius: 4
        color: "#1a1f2b"
        border.width: 1
        border.color: parent.activeFocus ? "#8ab4ff" : "#7aa2f7"
    }
}
