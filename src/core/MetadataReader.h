#pragma once

#include <QString>
#include <QByteArray>
#include <chrono>

namespace calyx::core {

struct Metadata {
    QString title;
    QString artist;
    QString album;
    QString genre;
    uint32_t trackNumber{0};
    uint32_t year{0};
    std::chrono::milliseconds duration{0};
    bool hasMetadata{false};
};

class MetadataReader {
public:
    static Metadata readMetadata(const QString& filePath);
    static QByteArray readCoverArt(const QString& filePath);
};

} // namespace calyx::core
