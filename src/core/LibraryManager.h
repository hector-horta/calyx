#pragma once

#include "core/Track.h"

#include <QObject>
#include <QDir>
#include <QVector>
#include <QString>

namespace calyx::core {

/// Manages the user's local music library: scanning, searching, and persistence.
class LibraryManager : public QObject {
    Q_OBJECT
public:
    explicit LibraryManager(QObject* parent = nullptr);
    ~LibraryManager() override = default;

    /// Recursively scan a directory for supported audio files.
    void scanDirectory(const QString& path);

    /// Return all tracks currently in the library.
    [[nodiscard]] QVector<Track> allTracks() const;

    /// Simple case-insensitive search across title, artist, and album.
    [[nodiscard]] QVector<Track> search(const QString& query) const;

    /// Number of tracks in the library.
    [[nodiscard]] int trackCount() const;

signals:
    void scanStarted();
    void scanFinished(int tracksFound);
    void trackAdded(const calyx::core::Track& track);

private:
    /// Attempt to read metadata from a file and create a Track.
    Track trackFromFile(const QString& filePath);

    QVector<Track> m_tracks;
    int64_t        m_nextId{1};

    static inline const QStringList s_supportedExtensions{
        "*.mp3", "*.flac", "*.m4a", "*.ogg", "*.wav", "*.aac"
    };
};

} // namespace calyx::core
