import QtQuick
import QtQuick.Controls

Item {
    id: root
    required property var playerBridge

    Column {
        anchors.fill: parent
        spacing: 20

        // Header
        Column {
            spacing: 8
            
            Text {
                text: qsTr("Apple Music Integration (Hello World)")
                color: "#ffffff"
                font.pointSize: 24
                font.bold: true
            }

            Text {
                text: qsTr("Prueba la comunicación asíncrona no-bloqueante con las APIs de Apple.")
                color: "#8e8e93"
                font.pointSize: 11
            }
        }

        // Configuration Panel (Token Input + Fetch Button)
        Rectangle {
            width: parent.width
            height: 120
            radius: 12
            color: "#121217"
            border.color: "#1e1e24"
            border.width: 1

            Row {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 16

                Column {
                    width: parent.width - 160 - 16
                    spacing: 8
                    anchors.verticalCenter: parent.verticalCenter

                    Text {
                        text: qsTr("Developer JWT Token")
                        color: "#a9a9b3"
                        font.pointSize: 9
                        font.bold: true
                    }

                    TextField {
                        id: tokenField
                        width: parent.width
                        height: 38
                        placeholderText: qsTr("Dejar vacío para usar la API pública de iTunes Search (Recomendado para pruebas)")
                        placeholderTextColor: "#4d4d5a"
                        color: "#ffffff"
                        font.pointSize: 9
                        
                        background: Rectangle {
                            color: "#0a0a0d"
                            radius: 6
                            border.color: tokenField.activeFocus ? "#ff2d55" : "#1e1e24"
                            border.width: 1
                        }
                    }
                }

                // Query Button
                Button {
                    id: queryButton
                    width: 160
                    height: 38
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: 14
                    enabled: !root.playerBridge.isFetching

                    background: Rectangle {
                        color: parent.enabled ? (mouseArea.containsMouse ? "#ff4d6d" : "#ff2d55") : "#331219"
                        radius: 6
                        
                        Behavior on color {
                            ColorAnimation { duration: 150 }
                        }
                        
                        // Subtle spinner while loading
                        Rectangle {
                            width: 12
                            height: 12
                            radius: 6
                            color: "#ffffff"
                            anchors.left: parent.left
                            anchors.leftMargin: 16
                            anchors.verticalCenter: parent.verticalCenter
                            visible: root.playerBridge.isFetching
                            
                            // Spin animation (mocked via opacity pulsing for simplicity)
                            NumberAnimation on opacity {
                                from: 0.2; to: 1.0; duration: 500; loops: Animation.Infinite; running: root.playerBridge.isFetching
                            }
                        }
                    }

                    contentItem: Text {
                        text: root.playerBridge.isFetching ? qsTr("Cargando...") : qsTr("Consultar API")
                        color: parent.enabled ? "#ffffff" : "#88ffffff"
                        font.bold: true
                        font.pointSize: 10
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        leftPadding: root.playerBridge.isFetching ? 24 : 0
                    }

                    onClicked: {
                        root.playerBridge.fetchAppleMusicCharts(tokenField.text)
                    }

                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: queryButton.clicked()
                        cursorShape: Qt.PointingHandCursor
                        enabled: queryButton.enabled
                    }
                }
            }
        }

        // Response View (Monospace terminal style)
        Column {
            width: parent.width
            height: parent.height - 220
            spacing: 8

            Text {
                text: qsTr("Respuesta JSON (Asíncrona):")
                color: "#a9a9b3"
                font.pointSize: 9
                font.bold: true
            }

            Rectangle {
                width: parent.width
                height: parent.height - 24
                radius: 12
                color: "#07070a"
                border.color: "#1e1e24"
                border.width: 1

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 16
                    clip: true

                    TextArea {
                        text: root.playerBridge.jsonResponse
                        color: "#a2d2ff" // Light blue color for JSON text
                        font.family: "Consolas"
                        font.pointSize: 9
                        readOnly: true
                        selectByMouse: true
                        wrapMode: Text.Wrap
                        background: null
                    }
                }
            }
        }
    }
}
