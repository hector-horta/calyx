#pragma once

#include <QString>
#include <QUrl>
#include <chrono>
#include <cstdint>

namespace calyx::core {

/// Immutable value object representing a single audio track.
struct Track {
    int64_t     id{0};
    QString     title;
    QString     artist;
    QString     album;
    QString     genre;
    uint32_t    trackNumber{0};
    uint32_t    year{0};
    std::chrono::milliseconds duration{0};
    QUrl        filePath;

    [[nodiscard]] bool isValid() const noexcept {
        return !filePath.isEmpty();
    }

    [[nodiscard]] QString durationFormatted() const {
        using namespace std::chrono;
        auto totalSec = duration_cast<seconds>(duration).count();
        auto min = totalSec / 60;
        auto sec = totalSec % 60;
        return QString("%1:%2").arg(min).arg(sec, 2, 10, QChar('0'));
    }

    bool operator==(const Track& other) const noexcept {
        return id == other.id && filePath == other.filePath;
    }
};

} // namespace calyx::core
