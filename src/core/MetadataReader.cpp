#include "core/MetadataReader.h"
#include <QFile>
#include <QFileInfo>
#include <QDir>
#include <QByteArray>
#include <QVector>
#include <algorithm>
#include <iostream>

namespace calyx::core {

static const char* s_id3v1Genres[] = {
    "Blues", "Classic Rock", "Country", "Dance", "Disco", "Funk", "Grunge", "Hip-Hop",
    "Jazz", "Metal", "New Age", "Oldies", "Other", "Pop", "R&B", "Rap", "Reggae", "Rock",
    "Techno", "Industrial", "Alternative", "Ska", "Death Metal", "Pranks", "Soundtrack",
    "Euro-Techno", "Ambient", "Trip-Hop", "Vocal", "Jazz+Funk", "Fusion", "Trance",
    "Classical", "Instrumental", "Acid", "House", "Game", "Sound Clip", "Gospel", "Noise",
    "Alternative Rock", "Bass", "Soul", "Punk", "Space", "Meditative", "Instrumental Pop",
    "Instrumental Rock", "Ethnic", "Gothic", "Darkwave", "Techno-Industrial", "Electronic",
    "Pop-Folk", "Eurodance", "Dream", "Southern Rock", "Comedy", "Cult", "Gangsta",
    "Top 40", "Christian Rap", "Pop/Funk", "Jungle", "Native US", "Cabaret", "New Wave",
    "Psychadelic", "Rave", "Showtunes", "Trailer", "Lo-Fi", "Tribal", "Acid Punk", "Acid Jazz",
    "Polka", "Retro", "Musical", "Rock & Roll", "Hard Rock"
};
constexpr int s_id3v1GenresCount = sizeof(s_id3v1Genres) / sizeof(s_id3v1Genres[0]);

static uint32_t readUint32BE(const char* data) {
    const auto* u = reinterpret_cast<const unsigned char*>(data);
    return (static_cast<uint32_t>(u[0]) << 24) |
           (static_cast<uint32_t>(u[1]) << 16) |
           (static_cast<uint32_t>(u[2]) << 8)  |
           static_cast<uint32_t>(u[3]);
}

static uint32_t readUint32Synchsafe(const char* data) {
    const auto* u = reinterpret_cast<const unsigned char*>(data);
    return (static_cast<uint32_t>(u[0]) << 21) |
           (static_cast<uint32_t>(u[1]) << 14) |
           (static_cast<uint32_t>(u[2]) << 7)  |
           static_cast<uint32_t>(u[3]);
}

static uint32_t readUint24BE(const char* data) {
    const auto* u = reinterpret_cast<const unsigned char*>(data);
    return (static_cast<uint32_t>(u[0]) << 16) |
           (static_cast<uint32_t>(u[1]) << 8)  |
           static_cast<uint32_t>(u[2]);
}

static QString parseId3TextFrame(const QByteArray& frameContent) {
    if (frameContent.size() < 2) return {};
    int encoding = frameContent[0];
    const char* dataPtr = frameContent.constData() + 1;
    int dataSize = frameContent.size() - 1;

    QString value;
    if (encoding == 0) {
        // ISO-8859-1 (Latin1)
        value = QString::fromLatin1(dataPtr, dataSize);
    } else if (encoding == 1) {
        // UTF-16 with BOM (byte order mark 0xFFFE or 0xFEFF)
        if (dataSize >= 2) {
            value = QString::fromUtf16(reinterpret_cast<const char16_t*>(dataPtr), dataSize / 2);
        }
    } else if (encoding == 2) {
        // UTF-16BE without BOM (convert BE to host endian)
        if (dataSize >= 2) {
            QVector<char16_t> buf(dataSize / 2);
            for (int i = 0; i < dataSize / 2; ++i) {
                unsigned char h = dataPtr[2 * i];
                unsigned char l = dataPtr[2 * i + 1];
                buf[i] = (h << 8) | l;
            }
            value = QString::fromUtf16(buf.constData(), buf.size());
        }
    } else if (encoding == 3) {
        // UTF-8
        value = QString::fromUtf8(dataPtr, dataSize);
    }

    value = value.trimmed();
    while (value.endsWith(QChar('\0'))) {
        value.chop(1);
    }
    return value;
}

static Metadata readMp3Metadata(QFile& file) {
    Metadata meta;
    if (!file.open(QIODevice::ReadOnly)) {
        return meta;
    }

    qint64 fileSize = file.size();

    // Read ID3v2 header
    char header[10];
    if (file.read(header, 10) == 10 && strncmp(header, "ID3", 3) == 0) {
        int majorVersion = header[3];
        uint32_t tagSize = readUint32Synchsafe(header + 6);

        // Read the entire tag data
        QByteArray tagData = file.read(tagSize);
        int offset = 0;
        while (offset + 10 < tagData.size()) {
            QString frameId;
            int frameSize = 0;
            int headerSize = 10;

            if (majorVersion == 2) {
                frameId = QString::fromLatin1(tagData.mid(offset, 3));
                frameSize = readUint24BE(tagData.constData() + offset + 3);
                headerSize = 6;
            } else {
                frameId = QString::fromLatin1(tagData.mid(offset, 4));
                if (majorVersion == 4) {
                    frameSize = readUint32Synchsafe(tagData.constData() + offset + 4);
                } else {
                    frameSize = readUint32BE(tagData.constData() + offset + 4);
                }
                headerSize = 10;
            }

            if (frameId.isEmpty() || frameId[0] == '\0' || frameSize <= 0 || offset + headerSize + frameSize > tagData.size()) {
                break;
            }

            QByteArray frameContent = tagData.mid(offset + headerSize, frameSize);
            offset += headerSize + frameSize;

            bool isTitle  = (frameId == "TIT2" || frameId == "TT2");
            bool isArtist = (frameId == "TPE1" || frameId == "TP1");
            bool isAlbum  = (frameId == "TALB" || frameId == "TAL");
            bool isGenre  = (frameId == "TCON" || frameId == "TCO");
            bool isTrack  = (frameId == "TRCK" || frameId == "TRK");
            bool isYear   = (frameId == "TYER" || frameId == "TYE" || frameId == "TDRC");

            if (isTitle || isArtist || isAlbum || isGenre || isTrack || isYear) {
                QString val = parseId3TextFrame(frameContent);
                if (!val.isEmpty()) {
                    meta.hasMetadata = true;
                    if (isTitle) meta.title = val;
                    else if (isArtist) meta.artist = val;
                    else if (isAlbum) meta.album = val;
                    else if (isGenre) meta.genre = val;
                    else if (isTrack) {
                        // Extract leading integer from string (e.g. "5/12" -> 5)
                        int slashIndex = val.indexOf('/');
                        if (slashIndex != -1) {
                            meta.trackNumber = val.left(slashIndex).toUInt();
                        } else {
                            meta.trackNumber = val.toUInt();
                        }
                    }
                    else if (isYear) {
                        // Extract year from date/year tag (e.g. "2020-04-12" -> 2020)
                        if (val.length() >= 4) {
                            meta.year = val.left(4).toUInt();
                        } else {
                            meta.year = val.toUInt();
                        }
                    }
                }
            }
        }
    }

    // Fallback to ID3v1 if necessary
    if (!meta.hasMetadata && fileSize >= 128) {
        if (file.seek(fileSize - 128)) {
            char v1Buf[128];
            if (file.read(v1Buf, 128) == 128 && strncmp(v1Buf, "TAG", 3) == 0) {
                meta.hasMetadata = true;
                meta.title = QString::fromLatin1(v1Buf + 3, 30).trimmed();
                meta.artist = QString::fromLatin1(v1Buf + 33, 30).trimmed();
                meta.album = QString::fromLatin1(v1Buf + 63, 30).trimmed();
                
                QString yearStr = QString::fromLatin1(v1Buf + 93, 4).trimmed();
                meta.year = yearStr.toUInt();

                // Genre is byte 127
                int genreId = static_cast<unsigned char>(v1Buf[127]);
                if (genreId >= 0 && genreId < s_id3v1GenresCount) {
                    meta.genre = QString::fromLatin1(s_id3v1Genres[genreId]);
                }

                // ID3v1.1 track number
                if (v1Buf[125] == 0 && v1Buf[126] != 0) {
                    meta.trackNumber = static_cast<unsigned char>(v1Buf[126]);
                }
            }
        }
    }

    file.close();
    return meta;
}

static Metadata readMp4Metadata(QFile& file) {
    Metadata meta;
    if (!file.open(QIODevice::ReadOnly)) {
        return meta;
    }

    qint64 fileSize = file.size();

    // Helper lambda to parse boxes recursively using QByteArray for robust byte-level matching
    auto parseBox = [&](auto& self, qint64 offset, qint64 limit, const QList<QByteArray>& path, int pathIndex) -> void {
        if (pathIndex >= path.size()) return;
        qint64 currentOffset = offset;
        while (currentOffset + 8 <= limit) {
            if (!file.seek(currentOffset)) return;
            char header[8];
            if (file.read(header, 8) != 8) return;

            quint32 boxSize = readUint32BE(header);
            QByteArray boxType(header + 4, 4);

            qint64 actualSize = boxSize;
            qint64 headerSize = 8;
            if (boxSize == 1) {
                char extHeader[8];
                if (file.read(extHeader, 8) != 8) return;
                actualSize = 0;
                const auto* uext = reinterpret_cast<const unsigned char*>(extHeader);
                for (int i = 0; i < 8; ++i) {
                    actualSize = (actualSize << 8) | uext[i];
                }
                headerSize = 16;
            } else if (boxSize == 0) {
                actualSize = fileSize - currentOffset;
            }

            if (actualSize < headerSize) return;

            if (boxType == path[pathIndex]) {
                qint64 contentOffset = currentOffset + headerSize;
                qint64 contentSize = actualSize - headerSize;

                if (pathIndex == path.size() - 1) {
                    // Target box found. Find 'data' sub-box
                    qint64 subOffset = contentOffset;
                    while (subOffset + 8 <= contentOffset + contentSize) {
                        if (!file.seek(subOffset)) break;
                        char subHeader[8];
                        if (file.read(subHeader, 8) != 8) break;
                        quint32 subSize = readUint32BE(subHeader);
                        QByteArray subType(subHeader + 4, 4);

                        if (subType == "data") {
                            if (subSize >= 16 && file.seek(subOffset + 16)) {
                                QByteArray strData = file.read(subSize - 16);
                                QByteArray targetType = path.last();
                                
                                if (targetType == "trkn") {
                                    if (strData.size() >= 6) {
                                        const auto* u = reinterpret_cast<const unsigned char*>(strData.constData());
                                        meta.trackNumber = (u[4] << 8) | u[5];
                                        meta.hasMetadata = true;
                                    }
                                } else {
                                    QString value = QString::fromUtf8(strData);
                                    value = value.trimmed();
                                    while (value.endsWith(QChar('\0'))) {
                                        value.chop(1);
                                    }
                                    if (!value.isEmpty()) {
                                        meta.hasMetadata = true;
                                        if (targetType == "\xA9" "nam") {
                                            meta.title = value;
                                        } else if (targetType == "sonm") {
                                            if (meta.title.isEmpty()) meta.title = value;
                                        } else if (targetType == "\xA9" "ART") {
                                            meta.artist = value;
                                        } else if (targetType == "soar") {
                                            if (meta.artist.isEmpty()) meta.artist = value;
                                        } else if (targetType == "\xA9" "alb") {
                                            meta.album = value;
                                        } else if (targetType == "soal") {
                                            if (meta.album.isEmpty()) meta.album = value;
                                        } else if (targetType == "\xA9" "gen") {
                                            meta.genre = value;
                                        } else if (targetType == "\xA9" "day") {
                                            if (value.length() >= 4) {
                                                meta.year = value.left(4).toUInt();
                                            } else {
                                                meta.year = value.toUInt();
                                            }
                                        }
                                    }
                                }
                            }
                            break;
                        }
                        subOffset += subSize;
                    }
                    return;
                } else {
                    qint64 subContentOffset = contentOffset;
                    qint64 subContentSize = contentSize;
                    if (boxType == "meta") {
                        subContentOffset += 4;
                        subContentSize -= 4;
                    }
                    self(self, subContentOffset, subContentOffset + subContentSize, path, pathIndex + 1);
                }
            }

            currentOffset += actualSize;
        }
    };

    // Scan for tags
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "\xA9" "nam"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "sonm"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "\xA9" "ART"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "soar"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "\xA9" "alb"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "soal"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "\xA9" "gen"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "\xA9" "day"}, 0);
    parseBox(parseBox, 0, fileSize, {"moov", "udta", "meta", "ilst", "trkn"}, 0);

    // Scan for duration inside mvhd box
    auto parseMvhd = [&](qint64 offset, qint64 limit) -> void {
        qint64 currentOffset = offset;
        while (currentOffset + 8 <= limit) {
            if (!file.seek(currentOffset)) return;
            char header[8];
            if (file.read(header, 8) != 8) return;
            quint32 boxSize = readUint32BE(header);
            QByteArray boxType(header + 4, 4);

            if (boxType == "moov") {
                qint64 subOffset = currentOffset + 8;
                qint64 subLimit = currentOffset + boxSize;
                while (subOffset + 8 <= subLimit) {
                    if (!file.seek(subOffset)) break;
                    char subHeader[8];
                    if (file.read(subHeader, 8) != 8) break;
                    quint32 subSize = readUint32BE(subHeader);
                    QByteArray subType(subHeader + 4, 4);

                    if (subType == "mvhd") {
                        if (file.seek(subOffset + 8)) {
                            char mvhdData[32];
                            if (file.read(mvhdData, 32) == 32) {
                                int version = mvhdData[0];
                                quint32 timescale = 0;
                                quint64 duration = 0;
                                if (version == 1) {
                                    const auto* udata = reinterpret_cast<const unsigned char*>(mvhdData + 16);
                                    timescale = (static_cast<quint32>(udata[0]) << 24) |
                                                (static_cast<quint32>(udata[1]) << 16) |
                                                (static_cast<quint32>(udata[2]) << 8)  |
                                                static_cast<quint32>(udata[3]);
                                    const auto* udur = reinterpret_cast<const unsigned char*>(mvhdData + 20);
                                    for (int i = 0; i < 8; ++i) {
                                        duration = (duration << 8) | udur[i];
                                    }
                                } else {
                                    const auto* udata = reinterpret_cast<const unsigned char*>(mvhdData + 8);
                                    timescale = (static_cast<quint32>(udata[0]) << 24) |
                                                (static_cast<quint32>(udata[1]) << 16) |
                                                (static_cast<quint32>(udata[2]) << 8)  |
                                                static_cast<quint32>(udata[3]);
                                    const auto* udur = reinterpret_cast<const unsigned char*>(mvhdData + 12);
                                    duration = (static_cast<quint32>(udur[0]) << 24) |
                                               (static_cast<quint32>(udur[1]) << 16) |
                                               (static_cast<quint32>(udur[2]) << 8)  |
                                               static_cast<quint32>(udur[3]);
                                }
                                if (timescale > 0) {
                                    meta.duration = std::chrono::milliseconds{(duration * 1000) / timescale};
                                }
                            }
                        }
                        break;
                    }
                    subOffset += subSize;
                }
                break;
            }
            currentOffset += boxSize;
        }
    };
    parseMvhd(0, fileSize);

    file.close();
    return meta;
}

Metadata MetadataReader::readMetadata(const QString& filePath) {
    QFileInfo fi(filePath);
    QString ext = fi.suffix().toLower();
    QFile file(filePath);

    if (ext == "mp3") {
        return readMp3Metadata(file);
    } else if (ext == "m4a" || ext == "m4p" || ext == "m4b" || ext == "mp4") {
        return readMp4Metadata(file);
    }

    return {};
}

} // namespace calyx::core
