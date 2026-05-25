import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    color: "#0d0d11"
    
    // Border to separate sidebar from main content
    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 1
        color: "#1e1e24"
    }

    property string activeView: "api_test"
    signal viewChanged(string viewName)

    // Brand Logo Area
    Item {
        id: logoArea
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 80
        
        Row {
            anchors.centerIn: parent
            spacing: 8
            
            // Decorative colored circle
            Rectangle {
                width: 12
                height: 12
                radius: 6
                color: "#ff2d55"
                anchors.verticalCenter: parent.verticalCenter
            }
            
            Text {
                text: "CALYX"
                color: "#ffffff"
                font.pointSize: 16
                font.bold: true
                font.letterSpacing: 2
                font.family: "System-UI"
            }
        }
    }

    // Navigation Items
    Column {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: logoArea.bottom
        anchors.topMargin: 20
        spacing: 6

        SidebarItem {
            text: qsTr("Listen Now")
            iconCharacter: "▶"
            viewId: "listen_now"
            active: root.activeView === "listen_now"
            onClicked: root.viewChanged("listen_now")
        }

        SidebarItem {
            text: qsTr("Browse")
            iconCharacter: "⊞"
            viewId: "browse"
            active: root.activeView === "browse"
            onClicked: root.viewChanged("browse")
        }

        SidebarItem {
            text: qsTr("Radio")
            iconCharacter: "📻"
            viewId: "radio"
            active: root.activeView === "radio"
            onClicked: root.viewChanged("radio")
        }

        SidebarItem {
            text: qsTr("Search")
            iconCharacter: "🔍"
            viewId: "search"
            active: root.activeView === "search"
            onClicked: root.viewChanged("search")
        }

        SidebarItem {
            text: qsTr("Local Library")
            iconCharacter: "📁"
            viewId: "local_library"
            active: root.activeView === "local_library"
            onClicked: root.viewChanged("local_library")
        }

        // Divider
        Rectangle {
            width: parent.width - 40
            height: 1
            color: "#1e1e24"
            anchors.horizontalCenter: parent.horizontalCenter
            visible: true
        }

        SidebarItem {
            text: qsTr("Apple Music API")
            iconCharacter: "⚡"
            viewId: "api_test"
            active: root.activeView === "api_test"
            onClicked: root.viewChanged("api_test")
        }
    }

    // Inline definition of SidebarItem
    component SidebarItem : Item {
        id: item
        property string text: ""
        property string iconCharacter: ""
        property string viewId: ""
        property bool active: false
        signal clicked()

        width: parent.width - 24
        height: 40
        anchors.horizontalCenter: parent.horizontalCenter

        Rectangle {
            id: bgRect
            anchors.fill: parent
            radius: 8
            color: item.active ? "#ff2d55" : (mouseArea.containsMouse ? "#1c1c24" : "transparent")
            
            Behavior on color {
                ColorAnimation { duration: 150 }
            }
        }

        // Active indicator dot
        Rectangle {
            width: 4
            height: 16
            radius: 2
            color: "#ffffff"
            anchors.left: parent.left
            anchors.leftMargin: 6
            anchors.verticalCenter: parent.verticalCenter
            visible: item.active
        }

        Row {
            anchors.fill: parent
            anchors.leftMargin: item.active ? 16 : 12
            spacing: 12
            
            Behavior on anchors.leftMargin {
                NumberAnimation { duration: 150; easing.type: Easing.OutQuad }
            }

            Text {
                text: item.iconCharacter
                color: item.active ? "#ffffff" : (mouseArea.containsMouse ? "#ffffff" : "#8e8e93")
                font.pointSize: 12
                anchors.verticalCenter: parent.verticalCenter
                
                Behavior on color {
                    ColorAnimation { duration: 150 }
                }
            }

            Text {
                text: item.text
                color: item.active ? "#ffffff" : (mouseArea.containsMouse ? "#ffffff" : "#a9a9b3")
                font.pointSize: 10
                font.bold: item.active
                anchors.verticalCenter: parent.verticalCenter

                Behavior on color {
                    ColorAnimation { duration: 150 }
                }
            }
        }

        MouseArea {
            id: mouseArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: item.clicked()
            cursorShape: Qt.PointingHandCursor
        }
    }
}
