import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs

Item {
    id: root
    property string viewName: "api_test"
    required property var playerBridge

    // State for local library sub-tab ("albums" or "songs")
    property string localLibraryTab: "albums"

    // Folder selection dialog for custom scanning
    FolderDialog {
        id: folderDialog
        title: qsTr("Select Music Folder")
        onAccepted: {
            root.playerBridge.scanFolder(selectedFolder.toString())
        }
    }

    // Content container
    Item {
        anchors.fill: parent
        anchors.margins: 32

        // 1. LISTEN NOW VIEW
        ScrollView {
            id: listenNowView
            anchors.fill: parent
            visible: root.viewName === "listen_now"
            
            Column {
                width: parent.width
                spacing: 24

                Text {
                    text: qsTr("Listen Now")
                    color: "#ffffff"
                    font.pointSize: 28
                    font.bold: true
                }

                Text {
                    text: qsTr("Up Next")
                    color: "#a9a9b3"
                    font.pointSize: 14
                    font.bold: true
                }

                // Grid of Albums
                Grid {
                    columns: Math.max(1, Math.floor(parent.width / 180))
                    spacing: 20
                    width: parent.width

                    AlbumCard {
                        title: "Future Nostalgia"
                        artist: "Dua Lipa"
                        colorCode: "#ff4d6d"
                        onPlayRequested: root.playerBridge.playTrack(title, artist)
                    }
                    AlbumCard {
                        title: "Blinding Lights"
                        artist: "The Weeknd"
                        colorCode: "#7209b7"
                        onPlayRequested: root.playerBridge.playTrack(title, artist)
                    }
                    AlbumCard {
                        title: "Random Access Memories"
                        artist: "Daft Punk"
                        colorCode: "#3f37c9"
                        onPlayRequested: root.playerBridge.playTrack(title, artist)
                    }
                    AlbumCard {
                        title: "Fine Line"
                        artist: "Harry Styles"
                        colorCode: "#4cc9f0"
                        onPlayRequested: root.playerBridge.playTrack(title, artist)
                    }
                }
            }
        }

        // 2. BROWSE VIEW
        ScrollView {
            anchors.fill: parent
            visible: root.viewName === "browse"

            Column {
                width: parent.width
                spacing: 24

                Text {
                    text: qsTr("Browse")
                    color: "#ffffff"
                    font.pointSize: 28
                    font.bold: true
                }

                Text {
                    text: qsTr("Featured Playlists")
                    color: "#a9a9b3"
                    font.pointSize: 14
                    font.bold: true
                }

                Grid {
                    columns: Math.max(1, Math.floor(parent.width / 180))
                    spacing: 20
                    width: parent.width

                    AlbumCard {
                        title: "Chill Vibes"
                        artist: "Apple Music Chill"
                        colorCode: "#06d6a0"
                        onPlayRequested: root.playerBridge.playTrack(title, artist)
                    }
                    AlbumCard {
                        title: "Today's Hits"
                        artist: "Apple Music Hits"
                        colorCode: "#ff2d55"
                        onPlayRequested: root.playerBridge.playTrack(title, artist)
                    }
                }
            }
        }

        // 3. RADIO VIEW
        ScrollView {
            anchors.fill: parent
            visible: root.viewName === "radio"

            Column {
                width: parent.width
                spacing: 24

                Text {
                    text: qsTr("Radio")
                    color: "#ffffff"
                    font.pointSize: 28
                    font.bold: true
                }

                // Apple Music 1 Live station
                Rectangle {
                    width: Math.min(500, parent.width)
                    height: 180
                    radius: 12
                    gradient: Gradient {
                        GradientStop { position: 0.0; color: "#ff2d55" }
                        GradientStop { position: 1.0; color: "#7209b7" }
                    }
                    
                    Column {
                        anchors.fill: parent
                        anchors.margins: 20
                        spacing: 8
                        
                        Text {
                            text: "LIVE STATION"
                            color: "#ffffff"
                            font.pointSize: 9
                            font.bold: true
                            opacity: 0.8
                        }
                        
                        Text {
                            text: "Apple Music 1"
                            color: "#ffffff"
                            font.pointSize: 22
                            font.bold: true
                        }
                        
                        Text {
                            text: "The new music that matters now."
                            color: "#ffffff"
                            font.pointSize: 11
                            opacity: 0.9
                        }

                        Item { width: 1; height: 10 }
                        
                        Button {
                            text: "Listen Now"
                            background: Rectangle {
                                color: "#ffffff"
                                radius: 16
                            }
                            contentItem: Text {
                                text: parent.text
                                font.bold: true
                                color: "#ff2d55"
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                            }
                            onClicked: root.playerBridge.playTrack("Apple Music 1 Live", "Apple Music")
                        }
                    }
                }
            }
        }

        // 4. SEARCH VIEW
        ScrollView {
            anchors.fill: parent
            visible: root.viewName === "search"

            Column {
                width: parent.width
                spacing: 24

                Text {
                    text: qsTr("Search")
                    color: "#ffffff"
                    font.pointSize: 28
                    font.bold: true
                }

                // Search field
                TextField {
                    id: searchField
                    width: Math.min(500, parent.width)
                    placeholderText: qsTr("Artists, Songs, Lyrics, and More")
                    placeholderTextColor: "#555562"
                    color: "#ffffff"
                    
                    background: Rectangle {
                        color: "#161622"
                        radius: 8
                        border.color: searchField.activeFocus ? "#ff2d55" : "#2c2c38"
                        border.width: 1
                    }
                }
            }
        }

        // 5. HELLO WORLD API VIEW
        ApiView {
            anchors.fill: parent
            visible: root.viewName === "api_test"
            playerBridge: root.playerBridge
        }

        // 6. LOCAL LIBRARY VIEW
        Item {
            anchors.fill: parent
            visible: root.viewName === "local_library"

            Column {
                anchors.fill: parent
                spacing: 20

                // Header & Action Bar
                Item {
                    width: parent.width
                    height: 40

                    // Left aligned: Title and view selector tabs
                    Row {
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 16

                        Text {
                            text: qsTr("Local Library")
                            color: "#ffffff"
                            font.pointSize: 28
                            font.bold: true
                            anchors.verticalCenter: parent.verticalCenter
                        }

                        Row {
                            spacing: 8
                            anchors.verticalCenter: parent.verticalCenter
                            leftPadding: 24

                            TabButton {
                                text: qsTr("Albums")
                                active: root.localLibraryTab === "albums"
                                onClicked: root.localLibraryTab = "albums"
                            }
                            
                            Text {
                                text: "|"
                                color: "#2c2c38"
                                font.pointSize: 12
                                anchors.verticalCenter: parent.verticalCenter
                            }

                            TabButton {
                                text: qsTr("Songs")
                                active: root.localLibraryTab === "songs"
                                onClicked: root.localLibraryTab = "songs"
                            }
                        }
                    }

                    // Right aligned: Action buttons
                    Row {
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 12

                        // Scan Custom Folder Button (Dark style with border)
                        Button {
                            id: chooseFolderBtn
                            text: qsTr("Scan Custom Folder...")
                            onClicked: folderDialog.open()
                            
                            background: Rectangle {
                                color: mouseAreaChoose.containsMouse ? "#2c2c35" : "#1e1e24"
                                radius: 19
                                border.color: "#3c3c45"
                                border.width: 1
                            }
                            contentItem: Text {
                                text: parent.text
                                color: "#ffffff"
                                font.bold: true
                                font.pointSize: 10
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: 16
                                rightPadding: 16
                            }
                            MouseArea {
                                id: mouseAreaChoose
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: chooseFolderBtn.clicked()
                                cursorShape: Qt.PointingHandCursor
                            }
                        }

                        // Scan Apple Music downloads folder (Pink brand style)
                        Button {
                            id: scanBtn
                            text: qsTr("Scan Apple Music Downloads")
                            onClicked: root.playerBridge.scanDefaultAppleMusicFolder()
                            
                            background: Rectangle {
                                color: mouseAreaScan.containsMouse ? "#ff4d6d" : "#ff2d55"
                                radius: 19
                            }
                            contentItem: Text {
                                text: parent.text
                                color: "#ffffff"
                                font.bold: true
                                font.pointSize: 10
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                                leftPadding: 16
                                rightPadding: 16
                            }
                            MouseArea {
                                id: mouseAreaScan
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: scanBtn.clicked()
                                cursorShape: Qt.PointingHandCursor
                            }
                        }
                    }
                }

                // SUB-VIEW 1: ALBUMS GRID GALLERY
                Rectangle {
                    width: parent.width
                    height: parent.height - 60
                    color: "transparent"
                    visible: root.localLibraryTab === "albums"

                    GridView {
                        id: albumsGrid
                        anchors.fill: parent
                        cellWidth: 180
                        cellHeight: 240
                        model: root.playerBridge.localAlbums
                        clip: true

                        delegate: AlbumCard {
                            title: modelData.albumName
                            artist: modelData.artistName
                            colorCode: modelData.colorCode
                            onPlayRequested: {
                                root.playerBridge.playAlbumTracks(modelData.tracks)
                            }
                        }

                        // Placeholder when empty
                        Text {
                            text: qsTr("No hay álbumes en tu biblioteca local.\nHaz clic en 'Scan Custom Folder...' para buscar archivos.")
                            color: "#6e6e73"
                            font.pointSize: 11
                            horizontalAlignment: Text.AlignHCenter
                            anchors.centerIn: parent
                            visible: albumsGrid.count === 0
                        }
                    }
                }

                // SUB-VIEW 2: SONGS DETAILED TABLE
                Column {
                    width: parent.width
                    height: parent.height - 60
                    spacing: 12
                    visible: root.localLibraryTab === "songs"

                    // Table Header
                    Rectangle {
                        width: parent.width
                        height: 30
                        color: "transparent"

                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12

                            Text { text: qsTr("Title"); color: "#6e6e73"; font.pointSize: 9; font.bold: true; width: parent.width * 0.4 }
                            Text { text: qsTr("Artist"); color: "#6e6e73"; font.pointSize: 9; font.bold: true; width: parent.width * 0.25 }
                            Text { text: qsTr("Album"); color: "#6e6e73"; font.pointSize: 9; font.bold: true; width: parent.width * 0.25 }
                            Text { text: qsTr("Duration"); color: "#6e6e73"; font.pointSize: 9; font.bold: true; width: parent.width * 0.1; horizontalAlignment: Text.AlignRight }
                        }
                    }

                    // List of local tracks
                    Rectangle {
                        width: parent.width
                        height: parent.height - 48
                        color: "#0a0a0d"
                        radius: 8
                        border.color: "#1e1e24"
                        border.width: 1

                        ListView {
                            id: localListView
                            anchors.fill: parent
                            anchors.margins: 4
                            model: root.playerBridge.localTracks
                            clip: true

                            delegate: Rectangle {
                                width: localListView.width
                                height: 44
                                color: mouseAreaTrack.containsMouse ? "#161622" : "transparent"
                                radius: 4

                                Row {
                                    anchors.fill: parent
                                    anchors.leftMargin: 12
                                    anchors.rightMargin: 12

                                    Text {
                                        text: modelData.title
                                        color: "#ffffff"
                                        font.pointSize: 10
                                        font.bold: true
                                        width: parent.width * 0.4
                                        anchors.verticalCenter: parent.verticalCenter
                                        elide: Text.ElideRight
                                    }

                                    Text {
                                        text: modelData.artist
                                        color: "#8e8e93"
                                        font.pointSize: 9.5
                                        width: parent.width * 0.25
                                        anchors.verticalCenter: parent.verticalCenter
                                        elide: Text.ElideRight
                                    }

                                    Text {
                                        text: modelData.album
                                        color: "#8e8e93"
                                        font.pointSize: 9.5
                                        width: parent.width * 0.25
                                        anchors.verticalCenter: parent.verticalCenter
                                        elide: Text.ElideRight
                                    }

                                    Text {
                                        text: modelData.durationFormatted
                                        color: "#8e8e93"
                                        font.pointSize: 9.5
                                        width: parent.width * 0.1
                                        anchors.verticalCenter: parent.verticalCenter
                                        horizontalAlignment: Text.AlignRight
                                    }
                                }

                                MouseArea {
                                    id: mouseAreaTrack
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    onClicked: {
                                        root.playerBridge.playLocalTrack(modelData.id)
                                    }
                                    cursorShape: Qt.PointingHandCursor
                                }
                            }

                            // Placeholder when empty
                            Text {
                                text: qsTr("No hay canciones en tu biblioteca local.\nHaz clic en 'Scan Custom Folder...' para buscar archivos.")
                                color: "#6e6e73"
                                font.pointSize: 11
                                horizontalAlignment: Text.AlignHCenter
                                anchors.centerIn: parent
                                visible: localListView.count === 0
                            }
                        }
                    }
                }
            }
        }
    }

    // Custom tab button for Albums/Songs switching
    component TabButton : Text {
        id: tabBtn
        property bool active: false
        signal clicked()

        color: tabBtn.active ? "#ff2d55" : (mouseAreaTab.containsMouse ? "#ffffff" : "#6e6e73")
        font.pointSize: 13
        font.bold: true
        anchors.verticalCenter: parent.verticalCenter
        
        Behavior on color {
            ColorAnimation { duration: 150 }
        }

        MouseArea {
            id: mouseAreaTab
            anchors.fill: parent
            hoverEnabled: true
            onClicked: tabBtn.clicked()
            cursorShape: Qt.PointingHandCursor
        }
    }

    // Beautiful Album Card Sub-component
    component AlbumCard : Item {
        id: card
        property string title: ""
        property string artist: ""
        property string colorCode: "#ffffff"
        signal playRequested()

        width: 160
        height: 220

        Column {
            anchors.fill: parent
            spacing: 8

            // Album cover artwork mock
            Rectangle {
                width: 160
                height: 160
                radius: 10
                color: card.colorCode
                
                // Hover overlay to show a play button
                Rectangle {
                    anchors.fill: parent
                    color: "#80000000"
                    radius: 10
                    opacity: mouseArea.containsMouse ? 1.0 : 0.0
                    
                    Behavior on opacity {
                        NumberAnimation { duration: 150 }
                    }

                    Text {
                        text: "▶"
                        color: "#ffffff"
                        font.pointSize: 24
                        anchors.centerIn: parent
                    }
                }

                // Smooth scaling on hover
                scale: mouseArea.containsMouse ? 1.03 : 1.0
                Behavior on scale {
                    NumberAnimation { duration: 150; easing.type: Easing.OutBack }
                }
            }

            Text {
                text: card.title
                color: "#ffffff"
                font.pointSize: 10
                font.bold: true
                width: parent.width
                elide: Text.ElideRight
            }

            Text {
                text: card.artist
                color: "#8e8e93"
                font.pointSize: 9
                width: parent.width
                elide: Text.ElideRight
            }
        }

        MouseArea {
            id: mouseArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: card.playRequested()
            cursorShape: Qt.PointingHandCursor
        }
    }
}
