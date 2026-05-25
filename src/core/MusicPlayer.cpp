#include "MusicPlayer.h"
#include "audio/QtAudioEngine.h"
#include <QDebug>
#include <QStandardPaths>
#include <QDir>
#include <QVariantMap>

namespace calyx::core {

MusicPlayer::MusicPlayer(QObject* parent)
    : QObject(parent)
    , m_currentTrackTitle("No Track Playing")
    , m_currentTrackArtist("Unknown Artist")
    , m_isPlaying(false)
    , m_jsonResponse("{}")
    , m_isFetching(false)
    , m_audioEngine(std::make_unique<audio::QtAudioEngine>(this))
    , m_player(std::make_unique<core::Player>(m_audioEngine.get(), this))
    , m_library(std::make_unique<core::LibraryManager>(this))
    , m_networkClient(std::make_unique<network::AppleMusicClient>(this))
{
    // Volume initial sync
    m_audioEngine->setVolume(m_volume);

    // Connect to network client events
    connect(m_networkClient.get(), &network::AppleMusicClient::requestStarted, this, [this]() {
        m_isFetching = true;
        emit isFetchingChanged(true);
    });

    connect(m_networkClient.get(), &network::AppleMusicClient::requestFinished, this, [this](const QString& json, bool success) {
        Q_UNUSED(success);
        m_isFetching = false;
        m_jsonResponse = json;
        emit isFetchingChanged(false);
        emit jsonResponseChanged(json);
    });

    // Connect to audio engine events
    connect(m_audioEngine.get(), &audio::IAudioEngine::stateChanged, this, [this](audio::IAudioEngine::State state) {
        bool playing = (state == audio::IAudioEngine::State::Playing);
        if (m_isPlaying != playing) {
            m_isPlaying = playing;
            emit isPlayingChanged(playing);
        }
    });

    connect(m_audioEngine.get(), &audio::IAudioEngine::positionChanged, this, [this](std::chrono::milliseconds pos) {
        m_position = static_cast<int>(pos.count());
        emit positionChanged();
    });

    connect(m_audioEngine.get(), &audio::IAudioEngine::durationChanged, this, [this](std::chrono::milliseconds dur) {
        m_duration = static_cast<int>(dur.count());
        emit durationChanged();
    });

    // Connect to library manager events
    connect(m_library.get(), &core::LibraryManager::scanFinished, this, [this](int count) {
        qDebug() << "Library scan finished, found" << count << "tracks";
        updateLocalTracks();
    });

    // Load initial empty list
    updateLocalTracks();
}

QString MusicPlayer::currentTrackTitle() const { return m_currentTrackTitle; }
QString MusicPlayer::currentTrackArtist() const { return m_currentTrackArtist; }
bool MusicPlayer::isPlaying() const { return m_isPlaying; }
QString MusicPlayer::jsonResponse() const { return m_jsonResponse; }
bool MusicPlayer::isFetching() const { return m_isFetching; }
QVariantList MusicPlayer::localTracks() const { return m_localTracks; }
QVariantList MusicPlayer::localAlbums() const { return m_localAlbums; }
int MusicPlayer::position() const { return m_position; }
int MusicPlayer::duration() const { return m_duration; }
int MusicPlayer::volume() const { return m_volume; }

void MusicPlayer::setCurrentTrackTitle(const QString& title) {
    if (m_currentTrackTitle != title) {
        m_currentTrackTitle = title;
        emit currentTrackTitleChanged(title);
    }
}

void MusicPlayer::setCurrentTrackArtist(const QString& artist) {
    if (m_currentTrackArtist != artist) {
        m_currentTrackArtist = artist;
        emit currentTrackArtistChanged(artist);
    }
}

void MusicPlayer::setIsPlaying(bool playing) {
    if (playing) {
        play();
    } else {
        pause();
    }
}

void MusicPlayer::setVolume(int vol) {
    if (m_volume != vol) {
        m_volume = vol;
        m_audioEngine->setVolume(vol);
        emit volumeChanged();
    }
}

void MusicPlayer::play() {
    m_player->play();
}

void MusicPlayer::pause() {
    m_player->pause();
}

void MusicPlayer::stop() {
    m_player->stop();
    setCurrentTrackTitle("No Track Playing");
    setCurrentTrackArtist("Unknown Artist");
}

void MusicPlayer::next() {
    m_player->next();
    auto cur = m_player->currentTrack();
    if (cur.has_value()) {
        setCurrentTrackTitle(cur->title);
        setCurrentTrackArtist(cur->artist.isEmpty() ? "Local File" : cur->artist);
    }
}

void MusicPlayer::previous() {
    m_player->previous();
    auto cur = m_player->currentTrack();
    if (cur.has_value()) {
        setCurrentTrackTitle(cur->title);
        setCurrentTrackArtist(cur->artist.isEmpty() ? "Local File" : cur->artist);
    }
}

void MusicPlayer::playTrack(const QString& title, const QString& artist) {
    setCurrentTrackTitle(title);
    setCurrentTrackArtist(artist);
    setIsPlaying(true);
}

void MusicPlayer::playLocalTrack(int trackId) {
    auto tracks = m_library->allTracks();
    for (const auto& t : tracks) {
        if (t.id == trackId) {
            m_player->playTrack(t);
            setCurrentTrackTitle(t.title);
            setCurrentTrackArtist(t.artist.isEmpty() ? "Local File" : t.artist);
            break;
        }
    }
}

void MusicPlayer::playAlbumTracks(const QVariantList& tracks) {
    if (tracks.isEmpty()) return;

    m_player->clearQueue();

    auto all = m_library->allTracks();
    for (const auto& var : tracks) {
        QVariantMap map = var.toMap();
        int trackId = map["id"].toInt();
        
        for (const auto& t : all) {
            if (t.id == trackId) {
                m_player->enqueue(t);
                break;
            }
        }
    }

    // Play the first track in queue
    m_player->next();
    auto cur = m_player->currentTrack();
    if (cur.has_value()) {
        setCurrentTrackTitle(cur->title);
        setCurrentTrackArtist(cur->artist.isEmpty() ? "Local File" : cur->artist);
    }
}

void MusicPlayer::seek(int ms) {
    m_player->seek(std::chrono::milliseconds{ms});
}

void MusicPlayer::fetchAppleMusicCharts(const QString& developerToken) {
    m_networkClient->fetchTestJSON(developerToken);
}

void MusicPlayer::scanFolder(const QString& folderPath) {
    QString localPath = folderPath;
    if (localPath.startsWith("file:///")) {
        localPath = QUrl(folderPath).toLocalFile();
    }
    qDebug() << "Scanning folder:" << localPath;
    m_library->scanDirectory(localPath);
}

void MusicPlayer::scanDefaultAppleMusicFolder() {
    // Check standard Windows Apple Music download paths:
    QString userMusic = QStandardPaths::writableLocation(QStandardPaths::MusicLocation);
    QString path1 = userMusic + "/Apple Music/Media/Apple Music";
    QString path2 = userMusic + "/Apple Music";
    QString path3 = userMusic + "/iTunes/iTunes Media/Apple Music";
    QString homeMusic = QDir::homePath() + "/Music";
    QString path4 = homeMusic + "/Apple Music/Media/Apple Music";
    QString path5 = homeMusic + "/Apple Music";
    QString path6 = homeMusic + "/iTunes/iTunes Media/Apple Music";

    if (QDir(path1).exists()) {
        scanFolder(path1);
    } else if (QDir(path2).exists()) {
        scanFolder(path2);
    } else if (QDir(path3).exists()) {
        scanFolder(path3);
    } else if (QDir(path4).exists()) {
        scanFolder(path4);
    } else if (QDir(path5).exists()) {
        scanFolder(path5);
    } else if (QDir(path6).exists()) {
        scanFolder(path6);
    } else {
        // Fallback: scan general Music folder if specific Apple Music one not found
        scanFolder(userMusic);
    }
}

void MusicPlayer::updateLocalTracks() {
    m_localTracks.clear();
    auto tracks = m_library->allTracks();
    for (const auto& t : tracks) {
        QVariantMap map;
        map["id"] = static_cast<int>(t.id);
        map["title"] = t.title;
        map["artist"] = t.artist.isEmpty() ? "Local File" : t.artist;
        map["album"] = t.album.isEmpty() ? "Unknown Album" : t.album;
        map["durationFormatted"] = t.durationFormatted();
        map["filePath"] = t.filePath.toString();
        m_localTracks.append(map);
    }
    emit localTracksChanged();
    
    // Group and update albums gallery
    updateLocalAlbums();
}

void MusicPlayer::updateLocalAlbums() {
    m_localAlbums.clear();
    
    // Grouping helper structure
    struct AlbumGroup {
        QString albumName;
        QString artistName;
        QVariantList tracks;
        QString colorCode;
    };
    
    QMap<QString, AlbumGroup> groups;
    auto tracks = m_library->allTracks();
    
    // Vibrant curated modern colors for card placeholders (Phase 1)
    QStringList colors = {"#ff2d55", "#7209b7", "#3f37c9", "#4cc9f0", "#06d6a0", "#ffb703", "#fb8500", "#d90429", "#8338ec", "#3a86c8"};

    for (const auto& t : tracks) {
        QString album = t.album;
        QString artist = t.artist;
        
        QFileInfo fi(t.filePath.toLocalFile());
        QDir dir = fi.dir();
        
        // If metadata is empty, extract from directory names
        if (album.isEmpty() || album == "Unknown Album") {
            album = dir.dirName(); // Parent directory is the album
        }
        
        if (artist.isEmpty() || artist == "Unknown Artist") {
            QDir parentDir = dir;
            if (parentDir.cdUp()) {
                artist = parentDir.dirName(); // Grandparent directory is the artist
                if (artist == "Apple Music" || artist == "Media" || artist == "Music") {
                    artist = "Local File";
                }
            } else {
                artist = "Local File";
            }
        }
        
        QString key = artist + "|" + album;
        
        QVariantMap trackMap;
        trackMap["id"] = static_cast<int>(t.id);
        trackMap["title"] = t.title;
        trackMap["artist"] = artist;
        trackMap["album"] = album;
        trackMap["durationFormatted"] = t.durationFormatted();
        trackMap["filePath"] = t.filePath.toString();

        if (!groups.contains(key)) {
            // Assign deterministic color based on hash of album name
            int colorIdx = qHash(album) % colors.size();
            groups[key] = AlbumGroup{album, artist, {trackMap}, colors[colorIdx]};
        } else {
            groups[key].tracks.append(trackMap);
        }
    }

    // Convert grouped map to flat list
    for (auto it = groups.begin(); it != groups.end(); ++it) {
        QVariantMap albumMap;
        albumMap["albumName"] = it.value().albumName;
        albumMap["artistName"] = it.value().artistName;
        albumMap["tracks"] = it.value().tracks;
        albumMap["colorCode"] = it.value().colorCode;
        albumMap["trackCount"] = it.value().tracks.size();
        m_localAlbums.append(albumMap);
    }
    
    emit localAlbumsChanged();
}

} // namespace calyx::core
