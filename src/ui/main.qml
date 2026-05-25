import QtQuick
import QtQuick.Controls
import Calyx 1.0

ApplicationWindow {
    id: window
    width: 1080
    height: 720
    minimumWidth: 800
    minimumHeight: 600
    visible: true
    title: qsTr("Calyx — Apple Music Client")

    // Custom dark background gradient
    background: Rectangle {
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#18181f" }
            GradientStop { position: 0.5; color: "#111115" }
            GradientStop { position: 1.0; color: "#0a0a0d" }
        }
    }

    // Instantiation of the C++ MusicPlayer class registered under Calyx namespace
    MusicPlayer {
        id: musicPlayer
    }

    // State for switching views ("listen_now", "browse", "radio", "search", "api_test")
    property string currentView: "api_test"

    // Root layout
    Item {
        anchors.fill: parent

        // Sidebar (Left panel)
        Sidebar {
            id: sidebar
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: playbackBar.top
            width: 230
            activeView: window.currentView

            onViewChanged: (viewName) => {
                window.currentView = viewName;
            }
        }

        // Main Content Area (Center / Right panel)
        MainContent {
            id: mainContent
            anchors.left: sidebar.right
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: playbackBar.top
            anchors.leftMargin: 1
            viewName: window.currentView
            playerBridge: musicPlayer
        }

        // Playback Control Bar (Bottom panel)
        PlaybackBar {
            id: playbackBar
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 88
            playerBridge: musicPlayer
        }
    }
}
