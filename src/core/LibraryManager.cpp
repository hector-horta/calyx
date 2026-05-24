#include "core/LibraryManager.h"

#include <QDirIterator>
#include <QFileInfo>
#include <QMediaMetaData>

namespace calyx::core {

LibraryManager::LibraryManager(QObject* parent)
    : QObject(parent)
{}

void LibraryManager::scanDirectory(const QString& path) {
    emit scanStarted();

    QDirIterator it(path, s_supportedExtensions,
                    QDir::Files, QDirIterator::Subdirectories);

    int found = 0;
    while (it.hasNext()) {
        it.next();
        Track t = trackFromFile(it.filePath());
        if (t.isValid()) {
            m_tracks.append(t);
            emit trackAdded(t);
            ++found;
        }
    }

    emit scanFinished(found);
}

QVector<Track> LibraryManager::allTracks() const {
    return m_tracks;
}

QVector<Track> LibraryManager::search(const QString& query) const {
    if (query.isEmpty()) return m_tracks;

    QVector<Track> results;
    const auto lowerQuery = query.toLower();

    for (const auto& t : m_tracks) {
        if (t.title.toLower().contains(lowerQuery) ||
            t.artist.toLower().contains(lowerQuery) ||
            t.album.toLower().contains(lowerQuery)) {
            results.append(t);
        }
    }
    return results;
}

int LibraryManager::trackCount() const {
    return static_cast<int>(m_tracks.size());
}

Track LibraryManager::trackFromFile(const QString& filePath) {
    QFileInfo fi(filePath);
    Track t;
    t.id       = m_nextId++;
    t.title    = fi.completeBaseName();   // Fallback: filename as title
    t.filePath = QUrl::fromLocalFile(filePath);
    return t;
}

} // namespace calyx::core
