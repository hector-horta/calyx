import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    color: "#121217"
    
    // Top border
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: "#1e1e24"
    }

    // Bridge to C++ MusicPlayer
    required property var playerBridge

    // Helper to format milliseconds to MM:SS
    function formatTime(ms) {
        if (ms <= 0) return "0:00";
        var totalSec = Math.floor(ms / 1000);
        var min = Math.floor(totalSec / 60);
        var sec = totalSec % 60;
        return min + ":" + (sec < 10 ? "0" + sec : sec);
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 24
        anchors.rightMargin: 24
        spacing: 20

        // 1. Current Track Details (Left third)
        Item {
            width: parent.width * 0.3
            height: parent.height
            
            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: 12
                
                // Track Artwork Container
                Rectangle {
                    width: 48
                    height: 48
                    radius: 6
                    color: "#1e1e24"
                    border.color: "#ff2d55"
                    border.width: 1
                    clip: true
                    visible: root.playerBridge.isPlaying

                    // Fallback emoji icon when cover is not loaded
                    Text {
                        text: "🎵"
                        anchors.centerIn: parent
                        font.pointSize: 18
                        opacity: 0.6
                        visible: coverImage.status !== Image.Ready
                    }

                    Image {
                        id: coverImage
                        anchors.fill: parent
                        source: root.playerBridge.currentTrackPath ? "image://covers/" + encodeURIComponent(root.playerBridge.currentTrackPath) : ""
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        visible: status === Image.Ready
                    }
                }
                
                // Track info text
                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4
                    
                    Text {
                        text: root.playerBridge.currentTrackTitle
                        color: "#ffffff"
                        font.pointSize: 10
                        font.bold: true
                        width: 220
                        elide: Text.ElideRight
                    }
                    
                    Text {
                        text: root.playerBridge.currentTrackArtist
                        color: "#8e8e93"
                        font.pointSize: 9
                        width: 220
                        elide: Text.ElideRight
                    }
                }
            }
        }

        // 2. Playback Controls & Progress (Center third)
        Item {
            width: parent.width * 0.4
            height: parent.height
            
            Column {
                anchors.centerIn: parent
                spacing: 6
                width: parent.width
                
                // Control Buttons
                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 24
                    
                    // Prev Button
                    PlaybackButton {
                        text: "⏮"
                        onClicked: root.playerBridge.previous()
                    }
                    
                    // Play/Pause Button
                    PlaybackButton {
                        text: root.playerBridge.isPlaying ? "⏸" : "▶"
                        isAccent: true
                        onClicked: {
                            if (root.playerBridge.isPlaying) {
                                root.playerBridge.pause()
                            } else {
                                root.playerBridge.play()
                            }
                        }
                    }
                    
                    // Next Button
                    PlaybackButton {
                        text: "⏭"
                        onClicked: root.playerBridge.next()
                    }
                }

                // Seek Slider using C++ values
                Row {
                    width: parent.width * 0.8
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 10
                    
                    Text {
                        text: root.formatTime(root.playerBridge.position)
                        color: "#6e6e73"
                        font.pointSize: 8
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    
                    Slider {
                        id: seekSlider
                        width: parent.width - 70
                        anchors.verticalCenter: parent.verticalCenter
                        from: 0
                        to: Math.max(1, root.playerBridge.duration)
                        value: root.playerBridge.position
                        
                        onMoved: {
                            root.playerBridge.seek(seekSlider.value)
                        }
                        
                        background: Rectangle {
                            x: seekSlider.leftPadding
                            y: seekSlider.topPadding + seekSlider.availableHeight / 2 - height / 2
                            implicitWidth: 200
                            implicitHeight: 4
                            width: seekSlider.availableWidth
                            height: implicitHeight
                            radius: 2
                            color: "#2c2c35"

                            Rectangle {
                                width: seekSlider.visualPosition * parent.width
                                height: parent.height
                                color: "#ff2d55"
                                radius: 2
                            }
                        }
                        
                        handle: Rectangle {
                            x: seekSlider.leftPadding + seekSlider.visualPosition * (seekSlider.availableWidth - width)
                            y: seekSlider.topPadding + seekSlider.availableHeight / 2 - height / 2
                            implicitWidth: 10
                            implicitHeight: 10
                            radius: 5
                            color: seekSlider.hovered ? "#ffffff" : "#ff2d55"
                            
                            Behavior on color {
                                ColorAnimation { duration: 100 }
                            }
                        }
                    }
                    
                    Text {
                        text: root.formatTime(root.playerBridge.duration)
                        color: "#6e6e73"
                        font.pointSize: 8
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }
        }

        // 3. Volume and Extra Settings (Right third)
        Item {
            width: parent.width * 0.3
            height: parent.height
            
            Row {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: 12
                
                Text {
                    text: "🔊"
                    color: "#8e8e93"
                    font.pointSize: 10
                    anchors.verticalCenter: parent.verticalCenter
                }
                
                Slider {
                    id: volumeSlider
                    width: 100
                    from: 0
                    to: 100
                    value: root.playerBridge.volume
                    anchors.verticalCenter: parent.verticalCenter
                    
                    onMoved: {
                        root.playerBridge.volume = volumeSlider.value
                    }
                    
                    background: Rectangle {
                        x: volumeSlider.leftPadding
                        y: volumeSlider.topPadding + volumeSlider.availableHeight / 2 - height / 2
                        implicitWidth: 100
                        implicitHeight: 4
                        width: volumeSlider.availableWidth
                        height: implicitHeight
                        radius: 2
                        color: "#2c2c35"

                        Rectangle {
                            width: volumeSlider.visualPosition * parent.width
                            height: parent.height
                            color: "#ffffff"
                            radius: 2
                            opacity: 0.8
                        }
                    }
                    
                    handle: Rectangle {
                        x: volumeSlider.leftPadding + volumeSlider.visualPosition * (volumeSlider.availableWidth - width)
                        y: volumeSlider.topPadding + volumeSlider.availableHeight / 2 - height / 2
                        implicitWidth: 10
                        implicitHeight: 10
                        radius: 5
                        color: "#ffffff"
                    }
                }
            }
        }
    }

    // Custom playback control button component
    component PlaybackButton : Rectangle {
        id: btn
        property string text: ""
        property bool isAccent: false
        signal clicked()

        width: btn.isAccent ? 36 : 30
        height: btn.isAccent ? 36 : 30
        radius: width / 2
        color: btn.isAccent ? "#ff2d55" : (mouseArea.containsMouse ? "#1c1c24" : "transparent")
        
        Behavior on color {
            ColorAnimation { duration: 150 }
        }

        Text {
            text: btn.text
            color: "#ffffff"
            font.pointSize: btn.isAccent ? 14 : 11
            anchors.centerIn: parent
        }

        MouseArea {
            id: mouseArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: btn.clicked()
            cursorShape: Qt.PointingHandCursor
        }
    }
}
