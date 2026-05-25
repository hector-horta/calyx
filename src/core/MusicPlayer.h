#pragma once

#include <QObject>
#include <QString>
#include <QVariantList>
#include <memory>
#include "audio/IAudioEngine.h"
#include "core/Player.h"
#include "core/LibraryManager.h"
#include "network/AppleMusicClient.h"

namespace calyx::core {

/// Bridge class between C++ logic (audio/network) and the QML frontend.
/// Manages both Apple Music API REST requests and local file playback/scanning.
class MusicPlayer : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString currentTrackTitle READ currentTrackTitle WRITE setCurrentTrackTitle NOTIFY currentTrackTitleChanged)
    Q_PROPERTY(QString currentTrackArtist READ currentTrackArtist WRITE setCurrentTrackArtist NOTIFY currentTrackArtistChanged)
    Q_PROPERTY(bool isPlaying READ isPlaying WRITE setIsPlaying NOTIFY isPlayingChanged)
    Q_PROPERTY(QString jsonResponse READ jsonResponse NOTIFY jsonResponseChanged)
    Q_PROPERTY(bool isFetching READ isFetching NOTIFY isFetchingChanged)
    Q_PROPERTY(QVariantList localTracks READ localTracks NOTIFY localTracksChanged)
    Q_PROPERTY(QVariantList localAlbums READ localAlbums NOTIFY localAlbumsChanged)
    Q_PROPERTY(int position READ position NOTIFY positionChanged)
    Q_PROPERTY(int duration READ duration NOTIFY durationChanged)
    Q_PROPERTY(int volume READ volume WRITE setVolume NOTIFY volumeChanged)

public:
    explicit MusicPlayer(QObject* parent = nullptr);
    ~MusicPlayer() override = default;

    // ── Property Getters ────────────────────────────────────────────────
    QString currentTrackTitle() const;
    QString currentTrackArtist() const;
    bool isPlaying() const;
    QString jsonResponse() const;
    bool isFetching() const;
    QVariantList localTracks() const;
    QVariantList localAlbums() const;
    int position() const;
    int duration() const;
    int volume() const;

    // ── Property Setters ────────────────────────────────────────────────
    void setCurrentTrackTitle(const QString& title);
    void setCurrentTrackArtist(const QString& artist);
    void setIsPlaying(bool playing);
    void setVolume(int vol);

    // ── QML Invokables (Playback Controls) ──────────────────────────────
    Q_INVOKABLE void play();
    Q_INVOKABLE void pause();
    Q_INVOKABLE void stop();
    Q_INVOKABLE void next();
    Q_INVOKABLE void previous();
    Q_INVOKABLE void playTrack(const QString& title, const QString& artist);
    Q_INVOKABLE void playLocalTrack(int trackId);
    Q_INVOKABLE void playAlbumTracks(const QVariantList& tracks);
    Q_INVOKABLE void seek(int ms);

    // ── QML Invokables (Network Layer) ──────────────────────────────────
    Q_INVOKABLE void fetchAppleMusicCharts(const QString& developerToken = "");

    // ── QML Invokables (Local Library Scanning) ─────────────────────────
    Q_INVOKABLE void scanFolder(const QString& folderPath);
    Q_INVOKABLE void scanDefaultAppleMusicFolder();

signals:
    void currentTrackTitleChanged(const QString& title);
    void currentTrackArtistChanged(const QString& artist);
    void isPlayingChanged(bool playing);
    void jsonResponseChanged(const QString& json);
    void isFetchingChanged(bool fetching);
    void localTracksChanged();
    void localAlbumsChanged();
    void positionChanged();
    void durationChanged();
    void volumeChanged();

private:
    QString m_currentTrackTitle;
    QString m_currentTrackArtist;
    bool m_isPlaying;
    QString m_jsonResponse;
    bool m_isFetching;
    QVariantList m_localTracks;
    QVariantList m_localAlbums;
    
    int m_position{0};
    int m_duration{0};
    int m_volume{75};

    std::unique_ptr<audio::IAudioEngine>       m_audioEngine;
    std::unique_ptr<core::Player>              m_player;
    std::unique_ptr<core::LibraryManager>      m_library;
    std::unique_ptr<network::AppleMusicClient> m_networkClient;

    void updateLocalTracks();
    void updateLocalAlbums();
};

} // namespace calyx::core
