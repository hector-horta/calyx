#pragma once

#include <QQuickImageProvider>

class CoverImageProvider : public QQuickImageProvider {
public:
    CoverImageProvider();
    ~CoverImageProvider() override = default;

    QImage requestImage(const QString& id, QSize* size, const QSize& requestedSize) override;
};
