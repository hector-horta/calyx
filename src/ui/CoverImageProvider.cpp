#include "CoverImageProvider.h"
#include "core/MetadataReader.h"
#include <QUrl>
#include <QImage>

CoverImageProvider::CoverImageProvider()
    : QQuickImageProvider(QQuickImageProvider::Image)
{
}

QImage CoverImageProvider::requestImage(const QString& id, QSize* size, const QSize& requestedSize) {
    Q_UNUSED(requestedSize);

    // The id is the absolute file path, which might be URL-encoded (e.g. from QML)
    // Decodes the URL component to get the clean absolute local file path
    QString filePath = QUrl::fromPercentEncoding(id.toUtf8());

    // If it starts with file:///, convert it properly
    if (filePath.startsWith("file:///")) {
        filePath = QUrl(filePath).toLocalFile();
    }

    QByteArray imgData = calyx::core::MetadataReader::readCoverArt(filePath);
    if (imgData.isEmpty()) {
        if (size) {
            *size = QSize(0, 0);
        }
        return QImage();
    }

    QImage img = QImage::fromData(imgData);
    if (img.isNull()) {
        if (size) {
            *size = QSize(0, 0);
        }
        return QImage();
    }

    if (size) {
        *size = img.size();
    }

    return img;
}
